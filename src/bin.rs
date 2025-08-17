mod args;

use anyhow::{Context, Result};
use chrono::Local;
use image::DynamicImage;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use satellite_imagery_downloader::download_image;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let args = args::parse();

    let prefs_path = std::env::current_dir()
        .context("Failed to get current dir")?
        .join(args.prefs);
    let mut prefs = get_prefrs(prefs_path)?;

    let prefs_dir = prefs["dir"].clone();
    let image_dir = Path::new(prefs_dir.as_str().unwrap());
    if !image_dir.exists() {
        fs::create_dir(image_dir).unwrap();
    }

    if let Some(url) = args.url {
        prefs["url"] = json!(url);
    }
    if let Some(tile_size) = args.tile_size {
        prefs["tile_size"] = json!(tile_size);
    }
    if let Some(out_dir) = args.out_dir {
        prefs["dir"] = json!(out_dir);
    }
    if let Some(top_left) = args.top_left {
        prefs["tl"] = json!(top_left);
    }
    if let Some(bottom_right) = args.bottom_right {
        prefs["br"] = json!(bottom_right);
    }
    if let Some(zoom) = args.zoom {
        prefs["zoom"] = json!(zoom);
    }

    let re = Regex::new(r"[+-]?\d*\.\d+|\d+").unwrap();
    let caps_tl: Vec<f64> = re
        .find_iter(prefs["tl"].as_str().unwrap())
        .map(|m| m.as_str().parse().unwrap())
        .collect();
    let caps_br: Vec<f64> = re
        .find_iter(prefs["br"].as_str().unwrap())
        .map(|m| m.as_str().parse().unwrap())
        .collect();

    if caps_tl.len() != 2 || caps_br.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid coordinates format. Expected two pairs of latitude and longitude."
        ));
    }

    if prefs["zoom"].is_null() {
        return Err(anyhow::anyhow!("Zoom level is required."));
    }

    let (lat1, lon1, lat2, lon2) = (caps_tl[0], caps_tl[1], caps_br[0], caps_br[1]);
    let zoom = prefs["zoom"].as_u64().unwrap() as u8;
    let tile_size = prefs["tile_size"].as_u64().unwrap() as u32;
    let channels = 4;
    let url_template = prefs["url"].as_str().unwrap();

    let mut headers = HeaderMap::new();
    if let Some(headers_map) = prefs["headers"].as_object() {
        for (key, value) in headers_map {
            if let Some(val_str) = value.as_str() {
                headers.insert(
                    key.parse::<HeaderName>().unwrap(),
                    HeaderValue::from_str(val_str).unwrap(),
                );
            }
        }
    }

    println!("Downloading image...");
    let img: DynamicImage = download_image(
        lat1,
        lon1,
        lat2,
        lon2,
        zoom,
        url_template,
        &headers,
        tile_size,
        channels,
    );
    println!("Downloaded successfully.");

    println!("Saving the image...");
    println!("Image size: {}x{}", img.width(), img.height());
    let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
    let name = format!("img_{timestamp}.png");
    let save_path = image_dir.join(&name);
    img.save(&save_path).unwrap();
    println!("Saved as {save_path:?}");

    if args.open {
        println!("Opening image in default viewer...");
        open::that_detached(save_path).context("Failed to open image in default viewer")?;
    }

    Ok(())
}

fn get_prefrs(path: PathBuf) -> Result<Value> {
    if !path.exists() {
        let files_dir = std::env::current_dir()
            .context("Failed to get current directory")?
            .join("images")
            .to_str()
            .context("Failed to convert path to a string")?
            .to_string();
        let default_prefs = json!({
            "url": "https://mt.google.com/vt/lyrs=s&x={x}&y={y}&z={z}",
            "tile_size": 256,
            "dir": files_dir,
            "headers": {
                "cache-control": "max-age=0",
                "sec-ch-ua": "\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"99\", \"Google Chrome\";v=\"99\"",
                "sec-ch-ua-mobile": "?0",
                "sec-ch-ua-platform": "\"Windows\"",
                "sec-fetch-dest": "document",
                "sec-fetch-mode": "navigate",
                "sec-fetch-site": "none",
                "sec-fetch-user": "?1",
                "upgrade-insecure-requests": "1",
                "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.4844.82 Safari/537.36"
            },
            "tl": "",
            "br": "",
            "zoom": ""
        });
        let serialized = serde_json::to_string_pretty(&default_prefs)
            .context("Failed to serialize default preferences")?;
        fs::write(&path, serialized).context("Failed to write default preferences file")?;
    }

    let content = fs::read_to_string(path).context("Failed to read preferences file")?;
    serde_json::from_str(&content).context("Failed to parse preferences file")
}
