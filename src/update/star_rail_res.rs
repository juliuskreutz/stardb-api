//! Synchronize source assets before refreshing the mtime-based WebP cache and pruning removed sources.

use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
    time::{Duration, Instant},
};

use actix_web::rt::{self, Runtime};
use anyhow::{anyhow, Result};
use image::{EncodableLayout, ImageFormat};

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

/// Synchronize the configured source before converting or pruning assets.
/// Git failures propagate without treating incomplete source state as upstream deletions.
async fn update_from(data_root: &Path, repo_url: &str) -> Result<()> {
    // Only a successful sync makes the source tree authoritative for conversion/pruning.
    // A failed clone or pull must not make missing source files look like upstream deletions.
    super::dimbreath::git_data::sync_data_repo(
        data_root
            .to_str()
            .ok_or_else(|| anyhow!("non-UTF8 asset root"))?,
        repo_url,
        "StarRailRes",
    )
    .await?;
    convert_assets(
        &data_root.join("StarRailRes"),
        &data_root.join("StarRailResWebp"),
    )
    .await
}

/// Re-encode PNGs newer than their WebP outputs, preserving relative paths and character-icon resizing.
/// Only after conversion succeeds, remove orphan WebPs; filesystem and image errors propagate.
async fn convert_assets(source_root: &Path, output_root: &Path) -> Result<()> {
    // Always scan: an interrupted conversion must recover even without a new commit.
    // Source mtimes invalidate existing outputs; mere output existence would retain
    // stale icons after git updates. Prune only after every conversion has succeeded.
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

            if new_path.exists()
                && fs::metadata(&new_path)?.modified()? >= fs::metadata(&path)?.modified()?
            {
                continue;
            }

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

            let encoder = Encoder::from_image(&png).map_err(|e| anyhow!("{e}"))?;
            let encoded_webp = encoder.encode_lossless();

            fs::write(new_path, encoded_webp.as_bytes())?;
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
    use std::{fs::FileTimes, path::PathBuf, process::Command, time::SystemTime};

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("stardb-assets-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn png(path: &Path, color: [u8; 4]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        image::RgbaImage::from_pixel(2, 2, image::Rgba(color))
            .save_with_format(path, ImageFormat::Png)
            .unwrap();
    }
    fn git(root: &Path, args: &[&str]) {
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
    }
    #[actix_web::test]
    async fn changed_png_is_reencoded_and_orphan_webp_is_removed() {
        let scratch = Scratch::new();
        let source = scratch.0.join("source");
        let output = scratch.0.join("output");
        fs::create_dir_all(source.join("image")).unwrap();
        let source_png = source.join("icon/avatar/test.png");
        let webp = output.join("icon/avatar/test.webp");
        png(&source_png, [255, 0, 0, 255]);
        convert_assets(&source, &output).await.unwrap();
        let first = fs::read(&webp).unwrap();
        // No wall-clock sleeps or filesystem timestamp-resolution assumptions.
        File::options()
            .write(true)
            .open(&webp)
            .unwrap()
            .set_times(FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))
            .unwrap();
        png(&source_png, [0, 0, 255, 255]);
        convert_assets(&source, &output).await.unwrap();
        assert_ne!(fs::read(&webp).unwrap(), first);
        let unrelated = output.join("keep.txt");
        fs::write(&unrelated, "keep").unwrap();
        fs::remove_file(&source_png).unwrap();
        convert_assets(&source, &output).await.unwrap();
        assert!(!webp.exists());
        assert!(unrelated.exists());
    }
    #[actix_web::test]
    async fn current_webp_is_not_reencoded() {
        let scratch = Scratch::new();
        let source = scratch.0.join("source");
        let output = scratch.0.join("output");
        fs::create_dir_all(source.join("image")).unwrap();
        let source_png = source.join("icon/avatar/test.png");
        png(&source_png, [255, 0, 0, 255]);
        convert_assets(&source, &output).await.unwrap();
        let webp = output.join("icon/avatar/test.webp");
        let before = fs::metadata(&webp).unwrap().modified().unwrap();
        // An invalid but older source must be skipped instead of decoded.
        fs::write(&source_png, "not a PNG").unwrap();
        File::options()
            .write(true)
            .open(&source_png)
            .unwrap()
            .set_times(FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))
            .unwrap();
        convert_assets(&source, &output).await.unwrap();
        assert_eq!(fs::metadata(&webp).unwrap().modified().unwrap(), before);
    }
    #[actix_web::test]
    async fn failed_local_git_pull_propagates_before_conversion() {
        let scratch = Scratch::new();
        let upstream = scratch.0.join("upstream");
        let data = scratch.0.join("static");
        fs::create_dir_all(&upstream).unwrap();
        git(&upstream, &["init", "-b", "main"]);
        png(&upstream.join("icon/avatar/test.png"), [255, 0, 0, 255]);
        fs::create_dir_all(upstream.join("image")).unwrap();
        fs::write(upstream.join("image/.gitkeep"), "").unwrap();
        git(&upstream, &["add", "."]);
        git(
            &upstream,
            &[
                "-c",
                "user.name=Asset Test",
                "-c",
                "user.email=asset@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-m",
                "fixture",
            ],
        );
        update_from(&data, upstream.to_str().unwrap())
            .await
            .unwrap();
        let webp = data.join("StarRailResWebp/icon/avatar/test.webp");
        let original = fs::read(&webp).unwrap();
        // Remote removal is entirely local and makes the subsequent pull fail.
        fs::remove_dir_all(&upstream).unwrap();
        fs::remove_file(data.join("StarRailRes/icon/avatar/test.png")).unwrap();
        let error = update_from(&data, upstream.to_str().unwrap())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("git pull failed"), "{error}");
        assert_eq!(
            fs::read(&webp).unwrap(),
            original,
            "failed sync must not prune or publish assets"
        );
    }
}
