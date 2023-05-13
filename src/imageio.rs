use log::{debug, info, trace};

use crate::args::*;

use image::{codecs::*, *};
use std::path::PathBuf;
use std::{fs::File, path::Path};

pub(crate) fn load_image(source: &PathBuf) -> ImageResult<DynamicImage> {
    let strpath = source.to_str().unwrap();
    info!("loading '{strpath}' ...");
    let imres = image::io::Reader::open(source)
        .expect("error while opening image")
        .with_guessed_format()
        .unwrap()
        .decode();
    debug!("done loading '{strpath}'");
    imres
}

pub(crate) fn save_image(
    img: &DynamicImage,
    path: &PathBuf,
    quality: &ImgQuality,
) -> ImageResult<()> {
    info!("saving image to '{}' ...", path.to_str().unwrap());
    match ImageFormat::from_path(path).expect("output is not a supported format!") {
        ImageFormat::Png => {
            trace!("saving as .png image");
            png::PngEncoder::new_with_quality(
                open_stream(path),
                match quality {
                    ImgQuality::Default => png::CompressionType::Default,
                    ImgQuality::Min => png::CompressionType::Fast,
                    ImgQuality::Max => png::CompressionType::Best,
                },
                png::FilterType::Adaptive,
            )
            .write_image(
                &img.to_rgba8(),
                img.width(),
                img.height(),
                image::ColorType::Rgba8,
            )
        }
        ImageFormat::Jpeg => {
            trace!("saving as .jpeg image");
            jpeg::JpegEncoder::new_with_quality(
                open_stream(path),
                match quality {
                    ImgQuality::Default => 70,
                    ImgQuality::Min => 30,
                    ImgQuality::Max => 100,
                },
            )
            .write_image(
                &img.to_rgba8(),
                img.width(),
                img.height(),
                image::ColorType::Rgba8,
            )
        }
        _ => {
            trace!("saving as generic image");
            img.save(path)
        }
    }
}

fn open_stream(path: &Path) -> File {
    trace!("opening stream to target location");
    File::create(path).expect("cannot open output file path")
}
