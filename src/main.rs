mod quad;
mod strct;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("missing paramenters!");
    }
    cald_and_draw(&args[1], &args[2]).unwrap()
}

pub fn cald_and_draw(file_in: &String, file_out: &String) -> Result<(),String>{

    let img = match image::io::Reader::open(file_in)
            .expect("error while opening image")
            .with_guessed_format()
            .unwrap()
            .decode()
            {
                Ok(img) => {
                    println!("image loaded successfully!"); img
                },
                Err(_) => 
                    return Err("Problem decoding image: {:?}".to_string()),
            };
        
    let img_out = quad::draw_quads_on_image(&img);

    img_out.save(file_out)
        .expect("error while saving image");

    Ok(())
}

pub fn panics(){panic!()}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(),String> {
        cald_and_draw(
            &String::from("tests/simple.png"), 
            &String::from("tests/output.png")
        )?;
        Ok(())
    }
}