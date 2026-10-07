use anyhow::{Result, anyhow};
use image::{DynamicImage, imageops::FilterType};
use rqrr::PreparedImage;

#[derive(Debug, Clone)]
pub struct QrDecodeOutput {
    pub content: String,
    pub version: usize,
    pub ecc_level: u16,
    pub mask: u16,
    pub module_count: usize,
    pub bounds: Vec<(i32, i32)>,
}

pub fn decode_image_bytes(bytes: &[u8]) -> Result<Vec<QrDecodeOutput>> {
    let img = image::load_from_memory(bytes).map_err(|_| anyhow!("无法识别该图片格式"))?;
    decode_dynamic(img)
}

pub fn decode_file(path: &str) -> Result<Vec<QrDecodeOutput>> {
    let bytes = std::fs::read(path).map_err(|e| anyhow!("读取图片失败：{e}"))?;
    decode_image_bytes(&bytes)
}

pub fn decode_matrix(matrix: &[Vec<bool>]) -> Result<Vec<QrDecodeOutput>> {
    let rows = matrix.len();
    if rows == 0 {
        return Err(anyhow!("矩阵为空"));
    }
    let cols = matrix[0].len();
    let scale: u32 = 4;
    let quiet: u32 = 4;
    let width = (cols as u32 + quiet * 2) * scale;
    let height = (rows as u32 + quiet * 2) * scale;
    let mut buf = image::GrayImage::from_pixel(width, height, image::Luma([255u8]));
    for (r, row) in matrix.iter().enumerate() {
        for (c, &on) in row.iter().enumerate() {
            if !on {
                continue;
            }
            let x0 = (c as u32 + quiet) * scale;
            let y0 = (r as u32 + quiet) * scale;
            for dy in 0..scale {
                for dx in 0..scale {
                    buf.put_pixel(x0 + dx, y0 + dy, image::Luma([0u8]));
                }
            }
        }
    }
    decode_dynamic(DynamicImage::ImageLuma8(buf))
}

fn decode_dynamic(img: DynamicImage) -> Result<Vec<QrDecodeOutput>> {
    let mut found = Vec::new();
    for (scale, invert) in [
        (1u32, false),
        (1u32, true),
        (2, false),
        (2, true),
        (3, false),
        (3, true),
    ] {
        found = scan(&img, scale, invert);
        if !found.is_empty() {
            break;
        }
    }
    if found.is_empty() {
        return Err(anyhow!(
            "未识别到二维码。请确认图像清晰、对比度足够、未被裁剪或严重倾斜"
        ));
    }
    Ok(found)
}

fn scan(img: &DynamicImage, scale: u32, invert: bool) -> Vec<QrDecodeOutput> {
    let mut luma = prepare(img, scale);
    if invert {
        for p in luma.pixels_mut() {
            p.0[0] = 255 - p.0[0];
        }
    }
    let mut prepared = PreparedImage::prepare(luma);
    let grids = prepared.detect_grids();
    grids.iter().filter_map(to_output).collect()
}

fn prepare(img: &DynamicImage, scale: u32) -> image::GrayImage {
    if scale <= 1 {
        img.to_luma8()
    } else {
        let (w, h) = (img.width() * scale, img.height() * scale);
        img.resize_exact(w, h, FilterType::Nearest).to_luma8()
    }
}

fn to_output<G: rqrr::BitGrid>(grid: &rqrr::Grid<G>) -> Option<QrDecodeOutput> {
    let (meta, content) = grid.decode().ok()?;
    Some(QrDecodeOutput {
        content,
        version: meta.version.0,
        ecc_level: meta.ecc_level,
        mask: meta.mask,
        module_count: grid.grid.size(),
        bounds: grid.bounds.iter().map(|p| (p.x, p.y)).collect(),
    })
}

pub fn ecc_name(level: u16) -> &'static str {
    match level {
        0 => "L",
        1 => "M",
        2 => "Q",
        3 => "H",
        _ => "未知",
    }
}
