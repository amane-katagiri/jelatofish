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

pub mod coswave;
pub mod spinflake;
pub mod flatwave;
pub mod rangefrac;
pub mod bubble;
pub mod test;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Debug)]
pub enum Generators {
    DEFAULT,
    Test,
    //Our first one is the workhorse Coswave. It can do anything.
    Coswave,
    //Next is the spinflake generator, for more shapely patterns.
    Spinflake,
    //The range fractal, which creates mountainous organic rough textures.
    //The flatwave generator, which creates interfering linear waves.
    //Bubble generator, which creates lumpy, curved turbulences.
}
impl Distribution<Generators> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Generators {
        match rng.gen_range(0..=1) {
            0 => Generators::Coswave,
            _ => Generators::Spinflake,
        }
    }
}

#[derive(Debug)]
pub struct GeneratorPropery {
    is_anti_aliased: bool,
    is_seamless: bool,
}

fn get_generator_property(generator: &Generators) -> GeneratorPropery {
    match generator {
        Generators::Coswave => GeneratorPropery {
            is_anti_aliased: false,
            is_seamless: false,
        },
        Generators::Spinflake => GeneratorPropery {
            is_anti_aliased: false,
            is_seamless: true,
        },
        _ => GeneratorPropery {
            is_anti_aliased: false,
            is_seamless: false,
        }
    }
}

#[derive(Debug)]
pub struct GeneratorParams {
    pub generator: Generators,
    pub coswave: coswave::CoswaveParams,
    pub spinflake: spinflake::SpinflakeParams,
}

#[derive(Debug)]
pub enum PackMethods {
    DEFAULT,
    ScaleToFit,
    FlipSignToFit,
    TruncateToFit,
    SlopeToFit,
}
impl Distribution<PackMethods> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PackMethods {
        match rng.gen_range(0..=3) {
            0 => PackMethods::ScaleToFit,
            1 => PackMethods::FlipSignToFit,
            2 => PackMethods::TruncateToFit,
            _ => PackMethods::SlopeToFit,
        }
    }
}
pub fn packed_cos(distance: f64, scale: f64, pack_method: &PackMethods) -> f64 {
    /*
    Many of the generators use a scheme where a wave is applied over
    a line. Since the range of a cosine wave is -1..0..1 rather than the
    simpler 0..1 expected by Starfish, we have to devise some way of packing
    the curve into the available range. These methods live in PackedCos, where
    they can be shared between all modules using such schemes.
    In addition, when new pack methods are devised, they can be added to the
    entire Starfish generator set simply by placing them in here.
    */
    let rawcos = (distance * scale).cos();
    match pack_method {
        //When the scale goes negative, turn it positive.
        PackMethods::FlipSignToFit => if rawcos >= 0.0 {rawcos} else {-rawcos},
        //When the scale goes negative, add 1 to it to bring it in range
        PackMethods::TruncateToFit => if rawcos >= 0.0 {rawcos} else {rawcos + 1.0},
        //Compress the -1..0..1 range of the normal cosine into 0..1
        PackMethods::ScaleToFit => (rawcos + 1.0) / 2.0,
        //use only the first half of the cycle. A saw-edge effect.
        PackMethods::SlopeToFit => ((distance * scale % std::f64::consts::PI).cos() + 1.0) / 2.0,
        _ => 0.5,
    }
}

pub fn generate(h: usize, v: usize, params: &GeneratorParams) -> Vec<Vec<f64>> {
    let mut rng = rand::thread_rng();

    let roll_h = rng.gen_range(0..=h);
    let roll_v = rng.gen_range(0..=v);

    vec![vec![0 as f64; h]; v].iter().enumerate().map(
        |(y, line)| {
            line.iter().enumerate().map(
                |(x, _)| {
                    f64::min(1.0, f64::max(0.0, get_layer_pixel(x, y, h, v, roll_h, roll_v, &params)))
                }
            ).collect()
        }
    ).collect()
}

fn get_layer_pixel(
    x: usize, y: usize, h: usize, v: usize, roll_h: usize, roll_v: usize, params: &GeneratorParams
) -> f64 {
    if x >= h && y >= v {
        return 0.0;
    }
    /*
    Calculate the point they wanted.
    Basically, we convert all of the coordinates into floating point
    values from 0 through 1. This lets the generators put out the same
    images regardless of the dimensions of the output data.
    Then we calculate the image, using the traditional old wrap/alias
    code. Then we convert the floating point value to a standard 0..255
    value and return it to the caller.
    */
    let x = (if x + roll_h < h {x + roll_h} else {x + roll_h - h}) as f64 / h as f64;
    let y = (if y + roll_v < v {y + roll_v} else {y + roll_v - v}) as f64 / v as f64;
    let fudge = 1.0 / (h + v) as f64;
    get_anti_aliased_point(x, y, fudge, params)
}

fn get_anti_aliased_point(x: f64, y: f64, fudge: f64, params: &GeneratorParams) -> f64 {
    let mut pixel = get_wrapped_point(x, y, params);
    if !get_generator_property(&params.generator).is_anti_aliased {
        /*
        This generator does not anti-alias itself.
        We need to do the anti-aliasing for it.
        The way we do this is to ask for a few more points, positioned
        between this point and the next one that will be computed.
        We then average all of these point values together. This does
        not affect the appearance of smooth gradients, but it significantly
        improves the way sharp transitions look. You can't see the individual
        pixels nearly so easily.
        */
        pixel += get_wrapped_point(x + fudge, y, params);
        pixel += get_wrapped_point(x, y + fudge, params);
        pixel += get_wrapped_point(x + fudge, y + fudge, params);
        pixel /= 4.0;
    }
    pixel
}

fn get_wrapped_point(x: f64, y: f64, params: &GeneratorParams) -> f64 {
    /*
    Get a point from this function.
    But don't just get the point - also get some out-of-band values and mix
    them in proportionately. This results in a seamlessly wrapped texture,
    where you can't see the edges.
    Some functions do this on their own; if that's the case, we let it do it.
    Otherwise, we do the computations ourself.
    */
    let mut pixel = call_generator(x, y, params);
    /*
    If this function does not generate seamlessly-tiled textures,
    then it is our job to pull in out-of-band data and mix it in
    with the actual pixel to get a smooth edge.
    */
    if !get_generator_property(&params.generator).is_seamless {
        /*
        We mix this pixel with out-of-band values from the opposite side
        of the tile. This is a "weighted average" proportionate to the pixel's
        distance from the edge of the tile. This creates a smoothly fading
        transition from one side of the texture to the other when the edges are
        tiled together.
        */
        //The farh and farv are on the opposite side of the tile.
        let farh = x + 1.0;
        let farv = y + 1.0;
        //There are three pixel values to grab off the edges.
        let farval1 = call_generator(x, farv, params);
        let farval2 = call_generator(farh, y, params);
        let farval3 = call_generator(farh, farv, params);
        //Calculate the weight factors for each far point.
        let weight = x * y;
        let farweight1 = x * (2.0 - farv);
        let farweight2 = (2.0 - farh) * y;
        let farweight3 = (2.0 - farh) * (2.0 - farv);
        let totalweight = weight + farweight1 + farweight2 + farweight3;
        //Now average all the pixels together, weighting each one by the local vs far weights.
        pixel = ((pixel * weight) + (farval1 * farweight1) + (farval2 * farweight2) + (farval3 * farweight3))
            / totalweight;
    }
    /*
    If the generator messes up and returns an out-of-range value, we clip it here.
    This way, curves that leap out of bounds simply get chopped off, instead of getting
    renormalized at the opposite end of the scale leading to big discontinuities and ugliness.
    This can mask bugs in a generator, but we aren't the generator so we don't care.
    If you're writing a generator it is your job to make your code work, and my job to
    make sure my code works even if yours doesn't.
    */
    if pixel > 1.0 {1.0} else if pixel < 0.0 {0.0} else {pixel}
}

fn call_generator(y: f64, x: f64, params: &GeneratorParams) -> f64 {
    match params.generator {
        Generators::Coswave
            => coswave::generate(y, x, &params.coswave),
        Generators::Spinflake
            => spinflake::generate(y, x, &params.spinflake),
        _ => test::generate(y, x),
    }
}
