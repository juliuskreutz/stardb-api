use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::{anyhow, Result};
use image::{EncodableLayout, ImageFormat};

use walkdir::WalkDir;
use webp::Encoder;

pub async fn star_rail_res() {
    rt::spawn(async move {
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
}

async fn update() -> Result<()> {
    if Command::new("git")
        .arg("pull")
        .current_dir("static/StarRailRes")
        .spawn()?
        .wait()
        .is_err()
    {
        Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/Mar-7th/StarRailRes",
            ])
            .current_dir("static")
            .spawn()?
            .wait()?;
    }

    for path in WalkDir::new("static/StarRailRes/icon")
        .into_iter()
        .chain(WalkDir::new("static/StarRailRes/image"))
        .flatten()
        .map(|e| e.into_path())
        .filter(|p| p.is_file())
    {
        if path.extension().and_then(|o| o.to_str()) == Some("png") {
            let mut new_path = PathBuf::from("static/StarRailResWebp")
                .join(path.strip_prefix("static/StarRailRes")?);
            new_path.set_extension("webp");

            if new_path.exists() {
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

    Ok(())
}
