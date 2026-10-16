//! Write frames to PNG, for headless screenshots and sprite sheets.

use super::gfx::{Frame, H, W};
use std::path::Path;

pub fn save_png(f: &Frame, path: &Path, scale: usize) {
    let (w, h) = (W as usize * scale, H as usize * scale);
    let mut data = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let c = f.px[(y / scale) * W as usize + x / scale];
            let i = (y * w + x) * 3;
            data[i] = (c >> 16) as u8;
            data[i + 1] = (c >> 8) as u8;
            data[i + 2] = c as u8;
        }
    }
    write_rgb(path, w, h, &data);
}

pub fn write_rgb(path: &Path, w: usize, h: usize, data: &[u8]) {
    let file = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(data).unwrap();
}

/// Tile several frames into one image (to judge animation).
pub fn save_sheet(frames: &[Frame], cols: usize, path: &Path, scale: usize) {
    let rows = frames.len().div_ceil(cols);
    let (fw, fh) = (W as usize * scale, H as usize * scale);
    let (w, h) = (fw * cols, fh * rows);
    let mut data = vec![0u8; w * h * 3];
    for (k, f) in frames.iter().enumerate() {
        let (ox, oy) = ((k % cols) * fw, (k / cols) * fh);
        for y in 0..fh {
            for x in 0..fw {
                let c = f.px[(y / scale) * W as usize + x / scale];
                let i = ((oy + y) * w + ox + x) * 3;
                data[i] = (c >> 16) as u8;
                data[i + 1] = (c >> 8) as u8;
                data[i + 2] = c as u8;
            }
        }
    }
    write_rgb(path, w, h, &data);
}

