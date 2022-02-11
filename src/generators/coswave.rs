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

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Debug)]
#[derive(Default)]
pub enum WaveAccelMethods {
    #[default]
    DEFAULT,
    None,
    Linear,
}


#[derive(Debug)]
#[derive(Default)]
pub struct CoswaveParams {
    origin: super::GeneratorPoint,
    wave_scale: f64,
    squish: f64,
    sqangle: f64,
    distortion: f64,
    pack_method: super::PackMethods,
    accel_method: WaveAccelMethods,
    accel: f64,
}
impl Distribution<CoswaveParams> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CoswaveParams {
        let mut params = CoswaveParams {
            origin: rand::random(),
            pack_method: rand::random(),
            wave_scale: rng.gen_range(0.0..=25.0) + 1.0,
            /*
            We don't like waves that are always perfect circles; they're too
            predictable. So we "squish" them a bit. We choose a squish factor, which
            serves as a multiplier. Currently wave scale modifications can range from
            half length to double length. It would be fun to widen this sometime and
            see what happened.
            The squish angle determines the "direction" of the squish effect. The
            strength of the squish is determined by the sine of the difference between
            the angle between the current point and the origin, and the sqangle.
            */
            sqangle: rng.gen_range(0.0..=std::f64::consts::PI),
            distortion: rng.gen_range(0.0..=1.5) + 0.5,
            squish: (rng.gen_range(0.0..=2.0) + 0.5)
                * if rng.gen_range(0..2) == 0 {1.0} else {-1.0},
            /* fill with default value (set later) */
            accel_method: WaveAccelMethods::DEFAULT,
            accel: 0.0,
        };

        /*
        I once attempted to make the coswave shift its scale over time, much like
        the spinflake generator does with its twist. I wasn't particularly succesful.
        But I did happen upon a *beautifully* bizarre twist to the generator which is
        really strange but not terribly useful. So I fire it once in every 64 generations
        or so, which is just infrequent enough that the viewer really goes "what the hell
        is THAT" when they see it.
        It's chaotic moirness, sorta - the wavescale increases by the exponent of the
        distance. At some point, the wavescale becomes less than one pixel, and then chaos
        begins to happen. Odd eddies show up, turbulences become visible, and a bit of static
        shines through here and there. It's quite beautiful in an abstract sort of way.
        */
        if rng.gen_range(0..64) == 0 {
            params.accel_method = WaveAccelMethods::Linear;
            params.accel = rng.gen_range(0.0..=2.0) + 1.0;
        } else {
            params.accel_method = WaveAccelMethods::None;
        }
        /*
        Packmethods flipsign and truncate effectively double the wavescale,
        because they turn both peaks and valleys into peaks. So we use a lower
        wavescale, then double it with the scaleToFit method to put it in range
        with the other packmethods.
        */
        if let super::PackMethods::ScaleToFit = params.pack_method {
            params.wave_scale *= 2.0;
        }

        params
    }
}

pub fn generate(pixel: super::GeneratorPoint, params: &CoswaveParams) -> f64 {
    //Rotate the axes of this shape.
    let x = pixel.x - params.origin.x;
    let y = pixel.y - params.origin.y;

    let hypangle = ((y / x) * params.distortion).atan() + params.sqangle;
    let hypotenuse = x.hypot(y);

    let x = hypangle.cos() * hypotenuse;
    let y = hypangle.sin() * hypotenuse;

    //Calculate the squished distance from the origin to the desired point.
    let hypotenuse = (x * params.squish).hypot(y / params.squish);
    //Scale the wavescale according to our accelerator function.
    let compwavescale = match params.accel_method {
        WaveAccelMethods::None => params.wave_scale,
        _ => params.wave_scale.powf(hypotenuse * params.accel),
    };
    let rawcos = super::packed_cos(hypotenuse, compwavescale, &params.pack_method);
    (rawcos + 1.0) / 2.0
}
