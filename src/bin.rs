use chrono::Local;
use image::DynamicImage;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use satellite_imagery_downloader::download_image;
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn take_input(messages: &[&str]) -> Option<Vec<String>> {
    let mut inputs = Vec::new();
    println!("Enter \"r\" to reset or \"q\" to exit.");
    for &message in messages {
        print!("{}", message);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_string();
        if input.eq_ignore_ascii_case("q") {
            return None;
        }
        if input.eq_ignore_ascii_case("r") {
            return take_input(messages);
        }
        inputs.push(input);
    }
    Some(inputs)
}

fn run() {
    let file_dir = std::env::current_dir().unwrap();
    let prefs_path = file_dir.join("preferences.json");

    let mut prefs: Value = if prefs_path.exists() {
        serde_json::from_str(&fs::read_to_string(&prefs_path).unwrap()).unwrap()
    } else {
        let default_prefs = json!({
            "url": "https://mt.google.com/vt/lyrs=s&x={x}&y={y}&z={z}",
            "tile_size": 256,
            "channels": 3,
            "dir": file_dir.join("images").to_str().unwrap().to_string(),
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
        fs::write(
            &prefs_path,
            serde_json::to_string_pretty(&default_prefs).unwrap(),
        )
        .unwrap();
        println!("Preferences file created in {:?}", prefs_path);
        return;
    };

    let prefs_dir = prefs["dir"].clone();
    let image_dir = Path::new(prefs_dir.as_str().unwrap());
    if !image_dir.exists() {
        fs::create_dir(image_dir).unwrap();
    }

    if prefs["tl"].as_str().unwrap().is_empty() || prefs["br"].as_str().unwrap().is_empty() {
        let messages = ["Top-left corner: ", "Bottom-right corner: ", "Zoom level: "];
        if let Some(inputs) = take_input(&messages) {
            prefs["tl"] = json!(inputs[0]);
            prefs["br"] = json!(inputs[1]);
            prefs["zoom"] = json!(inputs[2]);
        } else {
            return;
        }
    }

    if prefs["zoom"].as_str().unwrap().is_empty() {
        let messages = ["Zoom level: "];
        if let Some(inputs) = take_input(&messages) {
            prefs["zoom"] = json!(inputs[0]);
        } else {
            return;
        }
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

    let (lat1, lon1, lat2, lon2) = (caps_tl[0], caps_tl[1], caps_br[0], caps_br[1]);
    let zoom = prefs["zoom"].as_str().unwrap().parse::<u8>().unwrap();
    let tile_size = prefs["tile_size"].as_u64().unwrap() as u32;
    let channels = prefs["channels"].as_u64().unwrap() as u8;
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

    // Download the image
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

    // Save the image
    let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
    let name = format!("img_{}.png", timestamp);
    let save_path = image_dir.join(&name);
    img.save(&save_path).unwrap();
    println!("Saved as {:?}", save_path);
}

fn main() {
    run();
}
