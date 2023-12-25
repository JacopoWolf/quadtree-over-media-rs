use log::{debug, info, trace};

use crate::args::*;
use crate::drawing::apply_background_color;

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

pub(crate) fn load_background(drawarg: &DrawingArgs) -> Option<DynamicImage> {
    match drawarg.fill_with {
        Some(ref path) => match load_image(path) {
            Ok(img) => Some(if drawarg.background.is_some() {
                apply_background_color(&img, drawarg.background.as_ref().unwrap())
            } else {
                img
            }),
            Err(error) => panic!("problem opening fill-with image: {error:?}"),
        },
        None => None,
    }
}

pub(crate) fn save_image(
    img: &DynamicImage,
    path: &PathBuf,
    compression: &ImgCompression,
) -> ImageResult<()> {
    info!("saving image to '{}' ...", path.to_str().unwrap());

    match ImageFormat::from_path(path).expect("output is not a supported format!") {
        ImageFormat::Png => {
            trace!("saving as .png image");
            img.write_with_encoder(png::PngEncoder::new_with_quality(
                open_stream(path),
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
                open_stream(path),
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

fn open_stream(path: &Path) -> File {
    trace!("opening stream to target location");
    File::create(path).expect("cannot open output file path")
}
