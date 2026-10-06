use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use anyhow::{Error, Result};
use image::{
    DynamicImage, ImageFormat,
    codecs::{jpeg::JpegEncoder, pnm::PnmSubtype},
};

fn out_spec(name: &str) -> Result<(String, &'static str)> {
    match name.to_ascii_lowercase().as_str() {
        "png" => Ok(("png".to_string(), "png")),
        "jpg" | "jpeg" => Ok(("jpg".to_string(), "jpeg")),
        "webp" => Ok(("webp".to_string(), "webp")),
        "bmp" => Ok(("bmp".to_string(), "bmp")),
        "gif" => Ok(("gif".to_string(), "gif")),
        "tiff" | "tif" => Ok(("tiff".to_string(), "tiff")),
        "tga" => Ok(("tga".to_string(), "tga")),
        "ppm" => Ok(("ppm".to_string(), "ppm")),
        "pgm" => Ok(("pgm".to_string(), "pgm")),
        "pnm" => Ok(("pnm".to_string(), "pnm")),
        "hdr" => Ok(("hdr".to_string(), "hdr")),
        "exr" => Ok(("exr".to_string(), "exr")),
        "avif" => Ok(("avif".to_string(), "avif")),
        "qoi" => Ok(("qoi".to_string(), "qoi")),
        "ff" | "farbfeld" => Ok(("ff".to_string(), "ff")),
        other => Err(Error::msg(format!("不支持的输出格式: {other}"))),
    }
}

pub fn image_convert(
    inputs: &[String],
    output_format: &str,
    output_dir: &str,
    _quality: Option<u16>,
) -> Result<Vec<String>> {
    let (ext, kind) = out_spec(output_format)?;
    let dir = PathBuf::from(output_dir);
    std::fs::create_dir_all(&dir)?;

    let mut out_paths = Vec::with_capacity(inputs.len());
    for input in inputs {
        let in_path = Path::new(input);
        let img = image::open(in_path)
            .map_err(|e| Error::msg(format!("无法读取 {}: {e}", in_path.display())))?;
        let stem = in_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");
        let out_path = dir.join(format!("{stem}.{ext}"));
        write_image(&img, &out_path, kind)?;
        out_paths.push(out_path.to_string_lossy().into_owned());
    }
    Ok(out_paths)
}

fn write_image(img: &DynamicImage, path: &Path, kind: &str) -> Result<()> {
    match kind {
        "jpeg" => {
            let file = BufWriter::new(File::create(path)?);
            let mut enc = JpegEncoder::new_with_quality(file, 90);
            img.write_with_encoder(enc)
                .map_err(|e| Error::msg(e.to_string()))?;
        }
        "ppm" => {
            let file = BufWriter::new(File::create(path)?);
            let enc = image::codecs::pnm::PnmEncoder::new(file).with_subtype(PnmSubtype::Pixmap(
                image::codecs::pnm::SampleEncoding::Binary,
            ));
            img.to_rgb8()
                .write_with_encoder(enc)
                .map_err(|e| Error::msg(e.to_string()))?;
        }
        "pgm" => {
            let file = BufWriter::new(File::create(path)?);
            let enc = image::codecs::pnm::PnmEncoder::new(file).with_subtype(PnmSubtype::Graymap(
                image::codecs::pnm::SampleEncoding::Binary,
            ));
            img.to_luma8()
                .write_with_encoder(enc)
                .map_err(|e| Error::msg(e.to_string()))?;
        }
        _ => {
            let fmt = match kind {
                "png" => ImageFormat::Png,
                "webp" => ImageFormat::WebP,
                "bmp" => ImageFormat::Bmp,
                "gif" => ImageFormat::Gif,
                "tiff" => ImageFormat::Tiff,
                "tga" => ImageFormat::Tga,
                "pnm" => ImageFormat::Pnm,
                "hdr" => ImageFormat::Hdr,
                "exr" => ImageFormat::OpenExr,
                "avif" => ImageFormat::Avif,
                "qoi" => ImageFormat::Qoi,
                "ff" => ImageFormat::Farbfeld,
                _ => unreachable!(),
            };
            img.save_with_format(path, fmt)
                .map_err(|e| Error::msg(e.to_string()))?;
        }
    }
    Ok(())
}
