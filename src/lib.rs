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

extern crate wasm_bindgen;

pub mod game;
pub mod generators;
pub mod types;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::path::Path;
use wasm_bindgen::prelude::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct Colour {
    pub red: types::PixelVal,
    pub green: types::PixelVal,
    pub blue: types::PixelVal,
    pub alpha: types::PixelVal,
}
impl Colour {
    pub fn new(
        red: types::PixelVal,
        green: types::PixelVal,
        blue: types::PixelVal,
        alpha: types::PixelVal,
    ) -> Self {
        Colour {
            red,
            green,
            blue,
            alpha,
        }
    }
    pub fn scale(&self, factor: f64) -> Colour {
        Colour::new(
            self.red * factor,
            self.green * factor,
            self.blue * factor,
            self.alpha * factor,
        )
    }
}
impl Distribution<Colour> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Colour {
        Colour::new(
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
            0.0,
        )
    }
}

#[derive(Debug, Default)]
pub struct ColourPalette {
    pub colours: Vec<Colour>,
}
impl ColourPalette {
    pub fn sample(&self) -> Result<Colour, String> {
        /*
        Pick a random pixel from this palette.
        If the palette is empty, create it from random values.
        */
        if self.colours.len() > 1 {
            let mut rng = game::get_rng();

            let c = &self.colours[rng.gen_range(0..self.colours.len())];
            if 0.0 <= c.red
                && c.red <= 1.0
                && 0.0 <= c.green
                && c.green <= 1.0
                && 0.0 <= c.blue
                && c.blue <= 1.0
                && 0.0 <= c.alpha
                && c.alpha <= 1.0
            {
                return Ok(Colour::new(c.red, c.green, c.blue, c.alpha));
            }
            return Err("color values must be 0.0 <= r/g/b/a <= 1.0".to_string());
        }
        Ok(rand::random())
    }
}

#[derive(Debug)]
pub struct ColourLayer {
    //The image layer, a reference to pixels.
    image: types::PixelMap,
    //The foreground colour, used for high image values.
    fore: Colour,
    //The background colour, used for low image values.
    back: Colour,
    //The mask image. If None, we use the image layer as its own mask.
    mask: Option<types::PixelMap>,
    //If the flag is true, we invert the mask.
    invert_mask: bool,
}

#[derive(Debug)]
pub struct Jelatofish {
    size: types::Area,
    cutoff_threshold: types::PixelVal,
    layers: Vec<ColourLayer>,
}
impl Jelatofish {
    const MAX_LAYERS: usize = 6;
    const MIN_LAYERS: usize = 2;

    const MAX_CUTOFF_THRESHOLD: f64 = 1.0 / 16.0;

    pub fn random(
        size: types::Area,
        colours: &ColourPalette,
        layer_count: Option<usize>,
        cutoff_threshold: Option<types::PixelVal>,
    ) -> Result<Self, String> {
        /*
        Create a series of layers which we will later use to generate
        pixel data. These will contain the complete package of settings
        used to calculate image values.
        */
        let mut rng = game::get_rng();
        let layer_count = match layer_count {
            Some(x) if (Jelatofish::MIN_LAYERS..=Jelatofish::MAX_LAYERS).contains(&x) => x,
            None => rng.gen_range(Jelatofish::MIN_LAYERS..=Jelatofish::MAX_LAYERS),
            _ => {
                return Err(format!(
                    "must be {} <= layer_count <= {}",
                    Jelatofish::MIN_LAYERS,
                    Jelatofish::MAX_LAYERS,
                ))
            }
        };
        let cutoff_threshold = match cutoff_threshold {
            Some(x) if x <= Jelatofish::MAX_CUTOFF_THRESHOLD => x,
            None => rng.gen_range(0.0..=Jelatofish::MAX_CUTOFF_THRESHOLD),
            _ => {
                return Err(format!(
                    "must be cutoff_threshold <= {}",
                    Jelatofish::MAX_CUTOFF_THRESHOLD
                ))
            }
        };

        Ok(Jelatofish {
            size,
            cutoff_threshold,
            layers: vec![0; layer_count]
                .iter()
                .map(|_| {
                    /*
                    Now allocate random layers to use for the image and mask of this layer.
                    Half the time, we use the image as its own mask.
                    Half the time, we invert the mask.
                    */
                    //Now pick some random colours to use as fore and back of gradients.
                    let back = colours.sample().unwrap();
                    //The fore and back colours should NEVER be equal.
                    //Keep picking random colours until they don't match.
                    let fore = loop {
                        let fore = colours.sample().unwrap();
                        if fore.red != back.red
                            || fore.green != back.green
                            || fore.blue != back.blue
                        {
                            break fore;
                        }
                    };
                    let params: generators::GeneratorParams = rand::random();
                    ColourLayer {
                        image: generators::generate(size, &rand::random(), &params),
                        //Flip a coin. If it lands heads-up, create another layer for use as a mask.
                        mask: if game::maybe() {
                            Some(generators::generate(size, &rand::random(), &params))
                        } else {
                            None
                        },
                        //Flip another coin. If it lands heads-up, set the flag so we invert this layer.
                        invert_mask: game::maybe(),
                        back,
                        fore,
                    }
                })
                .collect(),
        })
    }
    pub fn get_pixel_val(&self, x: usize, y: usize) -> Result<Colour, String> {
        /*
        Calculate one pixel.
        We start with a black pixel.
        Then we loop through all of the layers, calculating each one with its
        mask. We then merge each layer's resulting pixel onto the out image.
        Once we're done, we return the merged pixel.
        We use alpha kind of backwards: high values mean high opacity, low values
        mean low opacity.
        */
        //Did we get valid parameters?
        if x >= self.size.width && y >= self.size.height {
            return Err(format!(
                "must be x >= {} && y >= {}",
                self.size.width, self.size.height
            ));
        }
        let mut outval: Colour = Default::default();
        for layer in &self.layers {
            //Get the image value for this pixel, for this layer.
            let imageval = layer.image[x][y];
            //Do we have a mask texture? If we do, calculate its value.
            let maskval = match &layer.mask {
                Some(mask) => mask[x][y],
                None => layer.image[x][y],
            };
            //Are we supposed to invert the mask value we got?
            let maskval = if layer.invert_mask {
                1.0 - maskval
            } else {
                maskval
            };
            /*
            Now we are ready. Calculate the image value for this layer.
            We use the image value as the proportion of the distance between
            two colours. We calculate this one channel at a time. This results
            in a smooth gradient of colour from min to max.
            */
            let mut layerpixel = Colour {
                red: imageval * (layer.fore.red - layer.back.red) + layer.back.red,
                green: imageval * (layer.fore.green - layer.back.green) + layer.back.green,
                blue: imageval * (layer.fore.blue - layer.back.blue) + layer.back.blue,
                alpha: maskval,
            };
            /*
            The image value for this layer is calculated.
            But the image is more than just this layer: it is the merged
            results of all the layers. So now we merge this value with
            the existing value calculated from the previous layers.
            We use the alpha channel to determine the proportion of blending.
            The new layer goes behind the existing layers; we use the existing
            alpha channel to determine what proportion of the new value shows
            through.
            */
            outval.red = (outval.red * outval.alpha) + (layerpixel.red * (1.0 - outval.alpha));
            outval.green =
                (outval.green * outval.alpha) + (layerpixel.green * (1.0 - outval.alpha));
            outval.blue = (outval.blue * outval.alpha) + (layerpixel.blue * (1.0 - outval.alpha));
            /*
            Add the alpha channels (representing opacity); if the result is greater
            than 100% opacity, we just stop calculating (since no further layers
            will produce visible data).
            */
            layerpixel.alpha *= 1.0 - outval.alpha;
            if layerpixel.alpha + outval.alpha + self.cutoff_threshold >= 1.0 {
                outval.alpha = 1.0;
                /*
                And now end the loop, because we've collected all the data we need.
                Calculating pixels from any of the deeper layers would just be a waste of time.
                */
                break;
            } else {
                outval.alpha += layerpixel.alpha;
            }
        }
        Ok(outval)
    }
}

#[wasm_bindgen]
pub fn new_fish_image() -> Box<[u8]> {
    let width = 256;
    let height = 256;
    let fish = Jelatofish::random(
        types::Area::new(width, height),
        &Default::default(),
        None,
        None,
    )
    .unwrap();
    const MAX_CHANVAL: f64 = 255.0;

    let image: Vec<u8> = vec![vec![0 as f64; width]; height]
        .iter()
        .enumerate()
        .flat_map(|(y, line)| {
            let line: Vec<u8> = line
                .iter()
                .enumerate()
                .flat_map(|(x, _)| {
                    let p = fish
                        .get_pixel_val(x, y)
                        .unwrap()
                        .scale(MAX_CHANVAL);
                    vec![p.red as u8, p.green as u8, p.blue as u8, 255]
                })
                .collect();
            line
        })
        .collect();
    image.into_boxed_slice()
}

pub fn save_test_image(
    width: usize,
    height: usize,
    generator: generators::Generators,
    filename: &str,
) {
    let image = generators::generate(types::Area::new(width, height), &generator, &rand::random());
    let mut imgbuf = image::ImageBuffer::new(width as u32, height as u32);

    const MAX_CHANVAL: f64 = 255.0;
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let p = image[x as usize][y as usize];
        *pixel = image::Rgb([
            (p * MAX_CHANVAL) as u8,
            (p * MAX_CHANVAL) as u8,
            (p * MAX_CHANVAL) as u8,
        ]);
    }
    imgbuf.save(&Path::new(filename)).unwrap();
}

pub fn save_fish_image(width: usize, height: usize, filename: &str) {
    let fish = Jelatofish::random(
        types::Area::new(width, height),
        &Default::default(),
        None,
        None,
    )
    .unwrap();
    let mut imgbuf = image::ImageBuffer::new(width as u32, height as u32);

    const MAX_CHANVAL: f64 = 255.0;
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let p = fish
            .get_pixel_val(x as usize, y as usize)
            .unwrap()
            .scale(MAX_CHANVAL);
        *pixel = image::Rgb([p.red as u8, p.green as u8, p.blue as u8]);
    }
    imgbuf.save(&Path::new(filename)).unwrap();
}
