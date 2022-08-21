mod quad;
mod utils;

use clap::Parser;
use image::Rgba;
use utils::rgba_from_colorname;


const DEFAULT_MIN_DEPTH: u32 = 4;
const DEFAULT_COLOR: Rgba<u8> = Rgba([255,20,147,255]); //DeepPink

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, short, value_parser, value_name = "IMAGE FILE")]
    input: String,

    #[clap(long, short, value_parser, value_name = "IMAGE FILE")]
    output: String,

    #[clap(long, value_parser, default_value_t = DEFAULT_MIN_DEPTH)]
    min_depth: u32,

    #[clap(long, short, value_parser = parse_rgba, value_name = "R,G,B,[A]|COLOR")]
    color: Option<Rgba<u8>>,

    #[clap(long, short = 'n', value_parser)]
    as_new: bool,
}


fn main() {
    let args = Args::parse();
    draw_and_save(&args)
}


pub fn draw_and_save(args: &Args){
    println!("loading {}", args.input);
    let img = match image::io::Reader::open(&args.input)
    .expect("error while opening image")
    .with_guessed_format()
    .unwrap()
    .decode()
    {
        Ok(img) => {
            println!("image loaded successfully!"); img
        },
        Err(_) => 
            panic!("Problem decoding image:"),
    };
    let img_out = quad::draw_quads_on_image(&img, &args);

    println!("saving image to {}", args.output);
    img_out
        .save(&args.output)
        .expect("error while saving image");
}

fn parse_rgba(s: &str) -> Result<Rgba<u8>, String> {
    if !s.contains(','){
        return Ok( rgba_from_colorname(&s) );
    }
    let v : Vec<u8> = s.split(',')
        .map(|p| p.parse::<u8>().expect("color value out of range"))
        .collect();
    match v.len() {
        3 => Ok(Rgba([v[0],v[1],v[2],255])),
        4 => Ok(Rgba([v[0],v[1],v[2],v[3]])),
        _ => Err("not r,g,b or r,g,b,a")?
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test] #[ignore]
    fn it_works() {
        draw_and_save(&Args{
            input: "tests/src/shapes.png".to_owned(),
            output: "tests/out/shapes.png".to_owned(),
            color: Option::Some(DEFAULT_COLOR),
            min_depth: DEFAULT_MIN_DEPTH,
            as_new: true
        })
    }
    #[test]
    fn parses_rgb() {
        assert_eq!(parse_rgba("250,251,252").unwrap(), Rgba([250,251,252,255]) )
    }
    #[test]
    fn parses_rgba() {
        assert_eq!(parse_rgba("64,65,66,128").unwrap(), Rgba([64,65,66,128]) )
    }
    #[test]
    fn parses_colorname() {
        assert_eq!(parse_rgba("red").unwrap(), Rgba([255,0,0,255]) )
    }
}
