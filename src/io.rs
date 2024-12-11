use crate::args::*;
use crate::drawing::apply_background_color;
use image::{codecs::*, *};
use log::{debug, info, trace};
use std::{fs::File, path::PathBuf};

pub(crate) fn load_image(source: &PathBuf) -> ImageResult<DynamicImage> {
    let strpath = source.to_str().unwrap();
    info!("loading image '{strpath}'");
    let imres = image::ImageReader::open(source)
        .expect("error while opening image")
        .with_guessed_format()?
        .decode();
    debug!("loaded image '{strpath}'");
    imres
}

pub(crate) fn load_filler(drawarg: &DrawingArgs) -> Result<Option<DynamicImage>, ImageError> {
    if let Some(ref path) = drawarg.fill_with {
        let img = load_image(path)?;
        Ok(Some(if let Some(bg) = drawarg.background {
            debug!("applying color to filler image");
            apply_background_color(&img, &bg)
        } else {
            img
        }))
    } else {
        Ok(None)
    }
}

pub(crate) fn save_image(
    img: &DynamicImage,
    path: &PathBuf,
    compression: &ImgCompression,
) -> ImageResult<()> {
    info!("saving image to '{}'", path.to_str().unwrap());

    match ImageFormat::from_path(path)? {
        ImageFormat::Png => {
            trace!("saving as .png image");
            img.write_with_encoder(png::PngEncoder::new_with_quality(
                File::create(path)?,
                match compression {
                    ImgCompression::Max => png::CompressionType::Best,
                    ImgCompression::High => png::CompressionType::Best,
                    ImgCompression::Default => png::CompressionType::Default,
                    ImgCompression::Low => png::CompressionType::Fast,
                    ImgCompression::No => png::CompressionType::Fast,
                },
                png::FilterType::Adaptive,
            ))
        }
        ImageFormat::Jpeg => {
            trace!("saving as .jpeg image");
            img.write_with_encoder(jpeg::JpegEncoder::new_with_quality(
                File::create(path)?,
                match compression {
                    ImgCompression::Max => 10,
                    ImgCompression::High => 40,
                    ImgCompression::Default => 70,
                    ImgCompression::Low => 82,
                    ImgCompression::No => 100,
                },
            ))
        }
        _ => {
            trace!("saving as generic image");
            img.save(path)
        }
    }
}
