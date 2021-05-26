use std::path::Path;
use image;

use jelatofish;

fn main() {
    let params = jelatofish::generators::GeneratorParams {
        coswave: jelatofish::generators::coswave::rand_param(),
        spinflake: jelatofish::generators::spinflake::rand_param(),
    };
    let pixels = jelatofish::generate(
        256, 256, jelatofish::generators::Generators::Spinflake, params
    );
    let mut imgbuf = image::ImageBuffer::new(256, 256);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let p = (pixels[y as usize][x as usize] * 255.0) as u8;
        *pixel = image::Rgb([p, p, p]);
    }
    imgbuf.save(&Path::new("image.png")).unwrap();
    println!("Hello, world!");
}
