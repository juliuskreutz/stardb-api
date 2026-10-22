//! Keep WebPs and one completed Git revision; source clones exist only during refresh.

use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
    time::{Duration, Instant},
};

use actix_web::rt::{self, Runtime};
use anyhow::{anyhow, Result};
use image::ImageFormat;

use walkdir::WalkDir;
use webp::Encoder;

/// Start immediate asset refreshes on a dedicated runtime, repeating every ten minutes and logging failures.
pub async fn spawn() {
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        let handle = rt.spawn(async move {
            let mut interval = rt::time::interval(Duration::from_secs(60 * 10));

            loop {
                interval.tick().await;

                let start = Instant::now();

                if let Err(e) = update().await {
                    error!(
                        "StarRailRes update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    info!(
                        "StarRailRes update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });

        rt.block_on(handle).unwrap();
    });
}

/// Refresh the production StarRailRes source and WebP output directories.
async fn update() -> Result<()> {
    update_from(
        Path::new("static"),
        "https://github.com/Mar-7th/StarRailRes",
    )
    .await
}

/// Skip unchanged revisions; rebuild changed revisions with a disposable shallow clone.
/// Invalidate success before touching outputs so partial failures and upstream reversions retry.
async fn update_from(data_root: &Path, repo_url: &str) -> Result<()> {
    use super::dimbreath::git_data;
    fs::create_dir_all(data_root)?;
    let source = data_root.join("StarRailRes");
    let output = data_root.join("StarRailResWebp");
    let marker = data_root.join(".StarRailResWebp-revision");
    // This reserved cache path is disposable, including leftovers from older versions.
    if source.exists() {
        fs::remove_dir_all(&source)?;
    }
    let latest = git_data::remote_head(repo_url, data_root).await?;
    if output.is_dir() && fs::read_to_string(&marker).is_ok_and(|old| old == latest) {
        return Ok(());
    }
    match fs::remove_file(&marker) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    // ponytail: rebuild every image per upstream commit; add per-file tracking only
    // if full refresh cost matters. Remove the marker to repair individual missing WebPs.
    let result = async {
        git_data::sync_data_repo(
            data_root
                .to_str()
                .ok_or_else(|| anyhow!("non-UTF8 asset root"))?,
            repo_url,
            "StarRailRes",
        )
        .await?;
        // A push between ls-remote and clone is fine: record what was actually converted.
        let revision = git_data::checkout_head(&source).await?;
        convert_assets(&source, &output).await?;
        write_atomic(&marker, revision.as_bytes())
    }
    .await;
    // Free PNGs and Git objects on failure too; the invalidated marker ensures retry.
    let cleanup = if source.exists() {
        fs::remove_dir_all(&source)
    } else {
        Ok(())
    };
    result?;
    cleanup?;
    Ok(())
}

/// Publish complete bytes via a sibling file so readers never see a partial WebP or marker.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let temporary = path.with_extension("tmp");
    let result = (|| {
        fs::write(&temporary, bytes)?;
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

/// Convert a complete checkout, preserving URLs, lossless encoding, and character-icon resizing.
/// Only after conversion succeeds, remove orphan WebPs; filesystem and image errors propagate.
async fn convert_assets(source_root: &Path, output_root: &Path) -> Result<()> {
    for path in WalkDir::new(source_root.join("icon"))
        .into_iter()
        .chain(WalkDir::new(source_root.join("image")))
    {
        let path = path?.into_path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|o| o.to_str()) == Some("png") {
            let mut new_path = output_root.join(path.strip_prefix(source_root)?);
            new_path.set_extension("webp");

            fs::create_dir_all(new_path.parent().unwrap())?;

            let mut png = image::load(BufReader::new(File::open(&path)?), ImageFormat::Png)?;

            if path.starts_with(source_root.join("icon/character")) {
                png = image::DynamicImage::ImageRgba8(image::imageops::resize(
                    &png,
                    128,
                    128,
                    image::imageops::FilterType::Lanczos3,
                ));
            }

            let rgba = png.to_rgba8();
            let encoder = Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
            let encoded_webp = encoder.encode_lossless();

            write_atomic(&new_path, &encoded_webp)?;
        }

        rt::task::yield_now().await;
    }

    if output_root.exists() {
        for entry in WalkDir::new(output_root) {
            let entry = entry?;
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|s| s.to_str()) == Some("webp")
            {
                let mut source = source_root.join(entry.path().strip_prefix(output_root)?);
                source.set_extension("png");
                if !source.exists() {
                    fs::remove_file(entry.path())?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }
    fn png(path: &Path, color: [u8; 4]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        image::RgbaImage::from_pixel(2, 2, image::Rgba(color))
            .save_with_format(path, ImageFormat::Png)
            .unwrap();
    }
    fn commit(root: &Path) {
        git(root, &["add", "."]);
        git(
            root,
            &["-c", "commit.gpgsign=false", "commit", "-m", "fixture"],
        );
    }

    #[actix_web::test]
    async fn revision_cache_skips_rebuilds_and_recovers_failed_partial_reversions() {
        let scratch = std::env::temp_dir().join(format!("stardb-assets-{}", uuid::Uuid::new_v4()));
        let upstream = scratch.join("upstream");
        let data = scratch.join("static");
        fs::create_dir_all(&upstream).unwrap();
        git(&upstream, &["init", "-b", "main"]);
        git(&upstream, &["config", "user.name", "Asset Test"]);
        git(
            &upstream,
            &["config", "user.email", "asset@example.invalid"],
        );
        let character = upstream.join("icon/character/test.png");
        png(&character, [255, 0, 0, 255]);
        png(&upstream.join("image/old.png"), [0, 255, 0, 255]);
        fs::write(upstream.join("image/.gitkeep"), "").unwrap();
        commit(&upstream);
        let repo = upstream.to_str().unwrap();
        let output = data.join("StarRailResWebp/icon/character/test.webp");
        let marker = data.join(".StarRailResWebp-revision");
        let source = data.join("StarRailRes");
        update_from(&data, repo).await.unwrap();
        let first = fs::read(&output).unwrap();
        let decoded = webp::Decoder::new(&first).decode().unwrap();
        assert_eq!((decoded.width(), decoded.height()), (128, 128));
        assert!(!source.exists());
        let modified = fs::metadata(&output).unwrap().modified().unwrap();
        update_from(&data, repo).await.unwrap();
        assert_eq!(fs::metadata(&output).unwrap().modified().unwrap(), modified);
        assert!(!source.exists());

        png(&character, [0, 0, 255, 255]);
        fs::remove_file(upstream.join("image/old.png")).unwrap();
        commit(&upstream);
        let good_revision = git(&upstream, &["rev-parse", "HEAD"]);
        update_from(&data, repo).await.unwrap();
        let good = fs::read(&output).unwrap();
        assert_ne!(good, first);
        assert!(!data.join("StarRailResWebp/image/old.webp").exists());
        assert_eq!(fs::read_to_string(&marker).unwrap(), good_revision);
        assert!(!source.exists());

        // Icon conversion happens before the image tree, guaranteeing a partial
        // published change before the invalid PNG fails the rest of the refresh.
        png(&character, [0, 255, 0, 255]);
        fs::write(upstream.join("image/bad.png"), "invalid PNG").unwrap();
        commit(&upstream);
        assert!(update_from(&data, repo).await.is_err());
        assert_ne!(fs::read(&output).unwrap(), good);
        assert!(!marker.exists());
        assert!(!source.exists());
        git(&upstream, &["reset", "--hard", &good_revision]);
        update_from(&data, repo).await.unwrap();
        assert_eq!(fs::read(&output).unwrap(), good);
        assert_eq!(fs::read_to_string(&marker).unwrap(), good_revision);
        assert!(!source.exists());
        fs::remove_dir_all(&upstream).unwrap();
        assert!(update_from(&data, repo).await.is_err());
        assert_eq!(fs::read(&output).unwrap(), good);
        assert!(!source.exists());
        fs::remove_dir_all(scratch).unwrap();
    }
}
