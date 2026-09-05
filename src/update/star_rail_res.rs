use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use actix_web::rt::{self, Runtime};
use anyhow::{anyhow, Result};
use image::{EncodableLayout, ImageFormat};

use walkdir::WalkDir;
use webp::Encoder;

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

async fn update() -> Result<()> {
    super::dimbreath::git_data::sync_data_repo(
        "static",
        "https://github.com/Mar-7th/StarRailRes",
        "StarRailRes",
    )
    .await?;
    // Always scan: an interrupted conversion must recover even without a new commit.
    for path in WalkDir::new("static/StarRailRes/icon")
        .into_iter()
        .chain(WalkDir::new("static/StarRailRes/image"))
    {
        let path = path?.into_path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|o| o.to_str()) == Some("png") {
            let mut new_path = PathBuf::from("static/StarRailResWebp")
                .join(path.strip_prefix("static/StarRailRes")?);
            new_path.set_extension("webp");

            if new_path.exists()
                && fs::metadata(&new_path)?.modified()? >= fs::metadata(&path)?.modified()?
            {
                continue;
            }

            fs::create_dir_all(new_path.parent().unwrap())?;

            let mut png = image::load(BufReader::new(File::open(&path)?), ImageFormat::Png)?;

            if path.starts_with("static/StarRailRes/icon/character/") {
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

    let output_root = Path::new("static/StarRailResWebp");
    if output_root.exists() {
        for entry in WalkDir::new(output_root) {
            let entry = entry?;
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|s| s.to_str()) == Some("webp")
            {
                let mut source =
                    Path::new("static/StarRailRes").join(entry.path().strip_prefix(output_root)?);
                source.set_extension("png");
                if !source.exists() {
                    fs::remove_file(entry.path())?;
                }
            }
        }
    }

    Ok(())
}
