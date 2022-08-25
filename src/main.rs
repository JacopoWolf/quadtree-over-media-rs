pub mod quad;
mod utils;

use clap::Parser;
use image::{DynamicImage, Rgba};


/// Calculate and draw quads over images, detecting "active" areas
/// and do nice stuff with that
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct QuadArgs {
    /// Location of input media. Can be any supported image
    #[clap(short, value_parser)]
    pub input: String,

    /// Location of output media
    #[clap(short, value_parser)]
    pub output: String,

    /// Minimun number of iterations
    #[clap(long, value_parser, default_value_t =  quad::DEFAULT_MIN_DEPTH)]
    pub min_depth: u32,

    /// The color of the lines defining the quads.
    /// Supports all CSS colors
    /// [default: "deeppink"]
    #[clap(long, short, value_parser = parse_color)]
    pub color: Option<Rgba<u8>>,

    /// The maximum allowed color difference between quadrants,
    /// ie: MAX(avgcolor)-MIN(avgcolor)
    /// Used to decide if to split or not. Passed as a CSS color value
    /// [default: "rgba(10,10,10,255)"]
    #[clap(long, short, value_parser = parse_color)]
    pub treshold: Option<Rgba<u8>>,

    /// fill the quads with the relative average color value
    #[clap(long, value_parser)]
    pub fill: bool,

    /// create the OUTPUT without drawing over a copy of INPUT media
    #[clap(long, value_parser)]
    pub quads_only: bool,
}

fn main() {
    let args = QuadArgs::parse();

    let img = quad::draw_quads_on_image(&load_image(&args.input), &args);
    save(&img, &args);
}

pub(crate) fn load_image(source: &String) -> DynamicImage {
    println!("loading {}", source);
    match image::io::Reader::open(&source)
        .expect("error while opening image")
        .with_guessed_format()
        .unwrap()
        .decode()
    {
        Ok(img) => {
            println!("image loaded successfully!");
            img
        }
        Err(_) => panic!("Problem decoding image:"),
    }
}

pub fn save(img: &DynamicImage, args: &QuadArgs) {
    println!("saving image to {}", args.output);
    img.save(&args.output).expect("error while saving image");
}

fn parse_color(s: &str) -> Result<Rgba<u8>, String> {
    match csscolorparser::parse(s) {
        Ok(c) => Ok(Rgba { 0: c.to_rgba8() }),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn it_works() {
        let args = &QuadArgs {
            input: "tests/src/shapes.png".to_owned(),
            output: "tests/out/shapes.png".to_owned(),
            color: parse_color("magenta").ok(),
            treshold: parse_color("#000").ok(),
            min_depth: 0,
            quads_only: true,
            fill: false,
        };
        let img = crate::quad::draw_quads_on_image(&load_image(&args.input), &args);
        save(&img, &args);
    }

    #[test]
    fn parses_rgba() {
        assert_eq!(
            parse_color("rgb(250,251,252)").unwrap(),
            Rgba([250, 251, 252, 255])
        );
        assert_eq!(parse_color("#a1b2c3").unwrap(), Rgba([161, 178, 195, 255]));
        assert_eq!(parse_color("#ff00007f").unwrap(), Rgba([255, 0, 0, 127]));
        assert_eq!(parse_color("red").unwrap(), Rgba([255, 0, 0, 255]));
    }
}
