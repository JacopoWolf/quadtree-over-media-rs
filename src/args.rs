use clap::Parser;
use image::Rgba;

use crate::quad;
use crate::utils::Vec2;

/// Calculate and draw quads over images, detecting color areas
/// to do nice stuff with that
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct QuadArgs {
    /// Location of input media. Can be any supported image
    #[clap(long, short, value_parser, value_name = "IMAGE")]
    pub input: String,

    /// Location of output media
    #[clap(long, short, value_parser, value_name = "IMAGE")]
    pub output: String,

    /// Minimun number of iterations
    #[clap(long, value_parser, default_value_t = quad::DEFAULT_MIN_DEPTH)]
    pub min_depth: u8,

    /// The minimum allowed size of a quad. Accepts any two number `x` `y`
    /// separated by an ascii punctuation character, ie: `[23,12]` `{55;56}` `4-2` `007=6`
    #[clap(long, value_parser = parse_vec2, default_value_t = quad::DEFAULT_MIN_SIZE)]
    pub min_quad_size: Vec2,

    /// The color of the lines defining the quads.
    /// Passed as a valid CSS color.
    /// [default: "deeppink"]
    #[clap(long, short, value_parser = parse_color)]
    pub color: Option<Rgba<u8>>,

    /// The maximum color difference between quadrants.
    /// A quadrant is split if the color difference is above this
    ///     value ie: `MAX(avgcolor)-MIN(avgcolor) > threshold`;
    /// a value of `0000` will always split, a value of `FFFF` will never split.
    /// For example if you want to split only on the Alpha channel use `FFF0`
    ///
    /// Passed as a valid CSS color.
    /// [default: rgba(10,10,10,255)]
    #[clap(long, short, value_parser = parse_color, value_name = "COLOR")]
    pub treshold: Option<Rgba<u8>>,

    /// fill the quads with the relative average color value.
    /// implies --no-drawover.
    #[clap(long, value_parser)]
    pub fill: bool,

    /// The image used to fill the quads.
    /// If `--fill` is also specified, it will multiply each pixel of this image
    /// by the average color of the quad.
    #[clap(long, value_parser, value_name = "IMAGE")]
    pub fill_with: Option<String>,

    /// create the OUTPUT without drawing over a copy of INPUT media
    #[clap(long, value_parser)]
    pub no_drawover: bool,

    /// when a new image is drawn this will be the backround color
    #[clap(long, short, value_parser = parse_color, value_name = "COLOR")]
    pub background: Option<Rgba<u8>>,

    /// the compression
    #[clap(long, arg_enum, default_value_t = ImgOpt::Default)]
    pub output_quality: ImgOpt,

    #[clap(long, value_parser = parse_color, value_name = "COLOR")]
    pub filter_gt: Option<Rgba<u8>>,
    #[clap(long, value_parser = parse_color, value_name = "COLOR")]
    pub filter_lt: Option<Rgba<u8>>,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ImgOpt {
    Default,
    Min,
    Max,
}

pub(super) fn parse_color(s: &str) -> Result<Rgba<u8>, String> {
    match csscolorparser::parse(s) {
        Ok(c) => Ok(Rgba(c.to_rgba8())),
        Err(e) => Err(e.to_string()),
    }
}

/// parses vec2. Supported formats: `x,y`, `x;y`, `[x,y]`
pub(super) fn parse_vec2(s: &str) -> Result<Vec2, String> {
    let split: Vec<&str> = s
        .split(|c: char| c.is_ascii_punctuation())
        .filter(|&p| !p.is_empty())
        .collect();
    if split.len() > 2 {
        return Err("not a vec2".to_owned());
    }
    let x: u32 = match split[0].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return Err("not a valid number".to_owned()),
    };
    let y: u32 = match split[1].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return Err("not a valid number".to_owned()),
    };
    let v = Vec2 { x, y };
    if v < quad::DEFAULT_MIN_SIZE {
        Err("min quad size is too small".to_owned())
    } else {
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use crate::{utils::Vec2, *};

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

    #[test]
    fn parses_vec2() {
        assert_eq!(parse_vec2("10,20"), Ok(Vec2 { x: 10, y: 20 }));
        assert_eq!(parse_vec2("-10-20-"), Ok(Vec2 { x: 10, y: 20 }));
        assert_eq!(parse_vec2("007=6"), Ok(Vec2 { x: 7, y: 6 }));
        assert_eq!(parse_vec2("(015,27)"), Ok(Vec2 { x: 15, y: 27 }));
        assert_eq!(parse_vec2("{264,664}"), Ok(Vec2 { x: 264, y: 664 }));
    }

    #[test]
    fn err_parsing_vec2() {
        assert_eq!(parse_vec2("10-11-12"), Err("not a vec2".to_owned()));
        assert_eq!(
            parse_vec2("2,2"),
            Err("min quad size is too small".to_owned())
        );
    }
}
