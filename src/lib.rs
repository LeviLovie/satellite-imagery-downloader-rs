use image::{DynamicImage, GenericImageView, RgbaImage};
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::sync::Mutex;

fn download_tile(
    url: &str,
    headers: &reqwest::header::HeaderMap,
    channels: u8,
) -> Option<DynamicImage> {
    let client = Client::new();
    if let Ok(response) = client.get(url).headers(headers.clone()).send() {
        if let Ok(bytes) = response.bytes() {
            if let Ok(img) = image::load_from_memory(&bytes) {
                return Some(if channels == 3 {
                    img.to_rgb8().into()
                } else {
                    img.to_rgba8().into()
                });
            }
        }
    }
    None
}

fn project_with_scale(lat: f64, lon: f64, scale: f64) -> (f64, f64) {
    let siny = lat.to_radians().sin().clamp(-0.9999, 0.9999);
    let x = scale * (0.5 + lon / 360.0);
    let y = scale * (0.5 - ((1.0 + siny) / (1.0 - siny)).ln() / (4.0 * std::f64::consts::PI));
    (x, y)
}

pub fn download_image(
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
    zoom: u8,
    url_template: &str,
    headers: &reqwest::header::HeaderMap,
    tile_size: u32,
    channels: u8,
) -> DynamicImage {
    let scale = 1 << zoom;
    let (tl_proj_x, tl_proj_y) = project_with_scale(lat1, lon1, scale as f64);
    let (br_proj_x, br_proj_y) = project_with_scale(lat2, lon2, scale as f64);

    let tl_pixel_x = (tl_proj_x * tile_size as f64) as i32;
    let tl_pixel_y = (tl_proj_y * tile_size as f64) as i32;
    let br_pixel_x = (br_proj_x * tile_size as f64) as i32;
    let br_pixel_y = (br_proj_y * tile_size as f64) as i32;

    let tl_tile_x = tl_proj_x as i32;
    let tl_tile_y = tl_proj_y as i32;
    let br_tile_x = br_proj_x as i32;
    let br_tile_y = br_proj_y as i32;

    let img_w = (br_pixel_x - tl_pixel_x).abs() as u32;
    let img_h = (br_pixel_y - tl_pixel_y).abs() as u32;

    let img = RgbaImage::new(img_w, img_h);
    let img = Mutex::new(img);
    let bar = Mutex::new(indicatif::ProgressBar::new(
        ((br_tile_x - tl_tile_x + 1) * (br_tile_y - tl_tile_y + 1)) as u64,
    ));

    (tl_tile_y..=br_tile_y).into_par_iter().for_each(|tile_y| {
        for tile_x in tl_tile_x..=br_tile_x {
            let url = url_template
                .replace("{x}", &tile_x.to_string())
                .replace("{y}", &tile_y.to_string())
                .replace("{z}", &zoom.to_string());
            if let Some(tile) = download_tile(&url, headers, channels) {
                let mut img = img.lock().unwrap();

                let tl_rel_x = tile_x * tile_size as i32 - tl_pixel_x;
                let tl_rel_y = tile_y * tile_size as i32 - tl_pixel_y;

                if tl_rel_x >= 0 && tl_rel_y >= 0 {
                    let tile_w = tile.width().min(img_w.saturating_sub(tl_rel_x as u32));
                    let tile_h = tile.height().min(img_h.saturating_sub(tl_rel_y as u32));

                    for y in 0..tile_h {
                        for x in 0..tile_w {
                            let pixel = tile.get_pixel(x, y);
                            img.put_pixel((tl_rel_x as u32) + x, (tl_rel_y as u32) + y, pixel);
                        }
                    }
                }
            }
            bar.lock().unwrap().inc(1);
        }
    });
    bar.lock().unwrap().finish();

    DynamicImage::ImageRgba8(img.into_inner().unwrap())
}

#[allow(dead_code)]
fn image_size(lat1: f64, lon1: f64, lat2: f64, lon2: f64, zoom: u8, tile_size: u32) -> (u32, u32) {
    let scale = 1 << zoom;
    let (tl_proj_x, tl_proj_y) = project_with_scale(lat1, lon1, scale as f64);
    let (br_proj_x, br_proj_y) = project_with_scale(lat2, lon2, scale as f64);

    let tl_pixel_x = (tl_proj_x * tile_size as f64) as u32;
    let tl_pixel_y = (tl_proj_y * tile_size as f64) as u32;
    let br_pixel_x = (br_proj_x * tile_size as f64) as u32;
    let br_pixel_y = (br_proj_y * tile_size as f64) as u32;

    (
        br_pixel_x.abs_diff(tl_pixel_x),
        br_pixel_y.abs_diff(tl_pixel_y),
    )
}
