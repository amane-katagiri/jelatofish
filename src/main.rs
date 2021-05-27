/*

Copyright ©2021 Amane Katagiri
Copyright ©1999 Mars Saxman
All Rights Reserved

This program is free software; you can redistribute it and/or
modify it under the terms of the GNU General Public License
as published by the Free Software Foundation; either version 2
of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA  02111-1307, USA.

*/

use jelatofish;

use std::path::Path;
use image;

fn main() {
    save_image(256, 256, "image.png");
    println!("Hello, world!");
}

pub fn save_image(width: usize, height: usize, filename: &str) {
    let params = jelatofish::generators::GeneratorParams {
        coswave: jelatofish::generators::coswave::rand_param(),
        spinflake: jelatofish::generators::spinflake::rand_param(),
    };
    let fish = jelatofish::Jelatofish::random(
        jelatofish::types::Area::new(width, height),
        &params, &Default::default(), None, None
    ).unwrap();
    let mut imgbuf = image::ImageBuffer::new(width as u32, height as u32);

    const MAX_CHANVAL: f64 = 255.0;
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let p = fish.get_pixel_val(x as usize, y as usize).unwrap();
        *pixel = image::Rgb([
            (p.red * MAX_CHANVAL) as u8,
            (p.blue * MAX_CHANVAL) as u8,
            (p.green * MAX_CHANVAL) as u8,
        ]);
    }
    imgbuf.save(&Path::new(filename)).unwrap();
}
