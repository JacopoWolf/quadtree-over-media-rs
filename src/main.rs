mod quad;
mod utils;

use clap::Parser;
use image::Rgba;

const DEFAULT_MIN_DEPTH : u32 = 4;
const DEFAULT_COLOR: Rgba<u8> = Rgba([255, 20, 147, 255]); //DeepPink
const DEFAULT_TRESHOLD: Rgba<u8> = Rgba([8, 8, 8, 255]);

/// Calculate and draw quads over images, detecting "active" areas
/// and do nice stuff with that
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Location of input media. Can be any supported image
    #[clap(short, value_parser)]
    input: String,

    /// Location of output media
    #[clap(short, value_parser)]
    output: String,

    /// Minimun number of iterations
    #[clap(long, value_parser, default_value_t = DEFAULT_MIN_DEPTH)]
    min_depth: u32,

    /// The color of the lines defining the quads.
    /// Supports all CSS colors
    /// [default: "deeppink"]
    #[clap(long, short, value_parser = parse_rgba)]
    color: Option<Rgba<u8>>,

    /// The maximum allowed color difference between quadrants, 
    /// ie: MAX(avgcolor)-MIN(avgcolor)
    /// Used to decide if to split or not. Passed as a CSS color value
    /// [default: "rgba(10,10,10,255)"]
    #[clap(long, short, value_parser = parse_rgba)]
    treshold: Option<Rgba<u8>>,

    /// fill the quads with the relative average color value
    #[clap(long, value_parser)]
    fill: bool,

    /// create the OUTPUT without drawing over a copy of INPUT media
    #[clap(long, value_parser)]
    quads_only: bool,
}

fn main() {
    let args = Args::parse();
    if args.treshold.is_some() && !args.treshold.unwrap().0.iter().all(|v| v > &0) {
        panic!("invalid treshold! minimum trashold must be >0 for each component!")
    }
    draw_and_save(&args)
}

pub fn draw_and_save(args: &Args) {
    println!("loading {}", args.input);
    let img = match image::io::Reader::open(&args.input)
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
    };
    let img_out = quad::draw_quads_on_image(&img, &args);

    println!("saving image to {}", args.output);
    img_out
        .save(&args.output)
        .expect("error while saving image");
}

fn parse_rgba(s: &str) -> Result<Rgba<u8>, String> {
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
        draw_and_save(&Args {
            input: "tests/src/shapes.png".to_owned(),
            output: "tests/out/shapes.png".to_owned(),
            color: Some(DEFAULT_COLOR),
            treshold: Some(Rgba([1,1,1,1])),
            min_depth: 0,
            quads_only: false,
            fill: false,
        })
    }
    #[test]
    fn parses_rgb() {
        assert_eq!(
            parse_rgba("rgb(250,251,252)").unwrap(),
            Rgba([250, 251, 252, 255])
        )
    }
    #[test]
    fn parses_rgb_hex() {
        assert_eq!(parse_rgba("#a1b2c3").unwrap(), Rgba([161, 178, 195, 255]))
    }

    #[test]
    fn parses_rgba_hex() {
        assert_eq!(parse_rgba("#ff00007f").unwrap(), Rgba([255,0,0,127]) )
    }

    #[test]
    fn parses_colorname() {
        assert_eq!(parse_rgba("red").unwrap(), Rgba([255, 0, 0, 255]))
    }
}
