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
pub enum SinePositivizingMethods {
    #[default]
    DEFAULT,
    CompressMethod,
    TruncateMethod,
    AbsoluteMethod,
    SawbladeMethod,
}
impl Distribution<SinePositivizingMethods> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SinePositivizingMethods {
        match rng.gen_range(0..=3) {
            0 => SinePositivizingMethods::CompressMethod,
            1 => SinePositivizingMethods::TruncateMethod,
            2 => SinePositivizingMethods::AbsoluteMethod,
            _ => SinePositivizingMethods::SawbladeMethod,
        }
    }
}


#[derive(Debug)]
#[derive(Default)]
pub enum TwirlMethods {
    #[default]
    DEFAULT,
    NoneMethod,
    CurveMethod,
    SineMethod,
}
impl Distribution<TwirlMethods> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TwirlMethods {
        match rng.gen_range(0..=2) {
            0 => TwirlMethods::NoneMethod,
            1 => TwirlMethods::CurveMethod,
            _ => TwirlMethods::SineMethod,
        }
    }
}


#[derive(Debug)]
#[derive(Default)]
pub struct Twirl {
    base: f64,
    speed: f64,
    amp: f64,
    method: TwirlMethods,
}
impl Twirl {
    const MAX_TWIRL: f64 = 14.0;
    const MAX_SINEAMP: f64 = 4.0;
}
impl Distribution<Twirl> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Twirl {
        let mut twirl = Twirl {
            base: rng.gen_range(0.0..=std::f64::consts::PI),
            method: rand::random(),
            ..Default::default()
        };
        match twirl.method {
            TwirlMethods::SineMethod => {
                twirl.speed = rng.gen_range(0.0..=(Twirl::MAX_TWIRL * std::f64::consts::PI));
                twirl.amp = rng.gen_range(-Twirl::MAX_SINEAMP..=Twirl::MAX_SINEAMP);
            },
            TwirlMethods::CurveMethod => {
                twirl.speed = rng.gen_range(-Twirl::MAX_TWIRL..=Twirl::MAX_TWIRL);
                twirl.amp = rng.gen_range(-Twirl::MAX_SINEAMP..=Twirl::MAX_SINEAMP);
            },
            _ => {},
        };
        twirl
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct Floret {
    sinepos_method: SinePositivizingMethods,
    backward: bool,
    spines: i32,
    spine_radius: f64,
    twirl: Twirl,
}
impl Distribution<Floret> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Floret {
        let mut floret = Floret{
            sinepos_method: rand::random(),
            backward: rng.gen_range(0..2) == 0,
            spines: rng.gen_range(0..=15) + 1,
            spine_radius: rng.gen_range(0.0..=0.5),
            twirl: rand::random(),
        };
        if let SinePositivizingMethods::AbsoluteMethod = floret.sinepos_method {
            if floret.spines % 2 == 1 {
                floret.spines += 1;
            }
        }
        floret
    }
}

#[derive(Debug)]
pub struct SpinflakeParams {
    origin: super::GeneratorPoint,
    radius: f64,
    squish: f64,
    twist: f64,
    average_florets: bool,
    layer: Vec<Floret>,
}
impl SpinflakeParams {
    const MAX_FLORETS: usize = 3;
}
impl Default for SpinflakeParams {
    fn default() -> Self {
        SpinflakeParams {
            layer: (0..1).map(|_| Default::default()).collect(),
            origin: Default::default(),
            radius: Default::default(),
            squish: Default::default(),
            twist: Default::default(),
            average_florets: Default::default(),
        }
    }
}
impl Distribution<SpinflakeParams> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SpinflakeParams {
        SpinflakeParams {
            origin: rand::random(),
            radius: rng.gen_range(0.0..=1.0),
            squish: rng.gen_range(0.0..=2.75) * 0.25,
            twist: rng.gen_range(0.0..=std::f64::consts::PI),
            average_florets: rng.gen_range(0..2) == 0,
            layer: (0..rng.gen_range(0..=(SpinflakeParams::MAX_FLORETS as i32)) + 1)
                .map(|_| rand::random()).collect(),
        }
    }
}

pub fn generate(pixel: super::GeneratorPoint, params: &SpinflakeParams) -> f64 {
    let val = vtiledpoint(pixel.x, pixel.y, params);
    if pixel.x > 0.5 {
        let farpoint = vtiledpoint(pixel.x - 1.0, pixel.y, params);
        let farweight = (pixel.x - 0.5) * 2.0;
        let weight = 1.0 - farweight;
        return (val * weight) + (farpoint * farweight);
    }
    val
}

fn chopsin(theta: f64, params: &Floret) -> f64 {
    let out = theta.sin();
    let out = match params.sinepos_method {
        SinePositivizingMethods::CompressMethod =>(out + 1.0) / 2.0,
        SinePositivizingMethods::AbsoluteMethod => out.abs(),
        SinePositivizingMethods::TruncateMethod => if out < 0.0 {out + 1.0} else {out},
        SinePositivizingMethods::SawbladeMethod => {
            let theta = theta / 4.0 % std::f64::consts::PI / 2.0;
            let theta = if theta < 0.0 {theta + (std::f64::consts::PI / 2.0)} else {theta};
            theta.sin()
        },
        _ => out,
    };
    if params.backward {
        return 1.0 - out;
    }
    out
}

fn vtiledpoint(x: f64, y: f64, params: &SpinflakeParams) -> f64 {
    let point = rawpoint(x, y, params);
    if y > 0.5 {
        let farpoint = rawpoint(x, y - 1.0, params);
        let farweight = (y - 0.5) * 2.0;
        let weight = 1.0 - farweight;
        return (point * weight) + (farpoint * farweight);
    }
    point
}

fn rawpoint(x: f64, y: f64, params: &SpinflakeParams) -> f64 {
    /*
    Rotate the point around our origin. This lets the squashed bulge-points on
    the sides of the squished spinflake point in random directions - not just aligned
    with the cartesian axes.
    */
    let x = x - params.origin.x;
    let y = y - params.origin.y;

    let hypangle = (y / x).atan() + params.twist;
    let origindist = x.hypot(y);

    let x = hypangle.cos() * origindist;
    let y = hypangle.sin() * origindist;
    //Calculate the distance from the origin to this point. Again.
    let origindist = (x * params.squish).hypot(y / params.squish);
    //If we are at the origin, there is no need to do the computations.
    if origindist != 0.0 {
        //The edge is (currently) a circle some radius units away.
        //Compute the angle this point represents to the origin.
        let pointangle = (y / x).atan();
        let mut edgedist = params.radius;
        for layer in &params.layer {
            edgedist += calcwave(pointangle, origindist, layer);
        }
        let edgedist =
            if params.average_florets {edgedist / (params.layer.len() as f64)} else {edgedist};
        //Our return value is the distance from the edge, proportionate
        //to the distance from the origin to the edge.
        let proportiondist = (edgedist - origindist) / edgedist;
        //If the value is >=0, we are inside the shape. Otherwise, we're outside it.
        if proportiondist >= 0.0 {
            return proportiondist.sqrt();
        } else {
            return 1.0 - (1.0 / (1.0 - proportiondist));
        }
    }
    1.0
}

fn calcwave(theta: f64, dist: f64, params: &Floret) -> f64 {
    /*
    Calculate the distance from centre this floret adds to the mix
    at the particular angle supplied.
    This is where we incorporate the floret's spines and twirling.
    Oddly, a spinflake's florets don't have to twirl in unison. This
    can get really interesting. If it doesn't work, migrate the twirl back
    to the spinflake instead.
    */
    let cosparam = match params.twirl.method {
        TwirlMethods::CurveMethod => theta * (params.spines as f64) + params.twirl.base
            + (dist * (params.twirl.speed + (dist * params.twirl.amp))),
        TwirlMethods::SineMethod => (theta * (params.spines as f64) + params.twirl.base)
            + ((dist * params.twirl.speed).sin() * (params.twirl.amp + (dist * params.twirl.amp))),
        _ => theta * (params.spines as f64) + params.twirl.base,
    };
    chopsin(cosparam, params) * params.spine_radius
}
