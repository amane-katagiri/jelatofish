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
pub enum InterferenceMethods {
    #[default]
    DEFAULT,
    MostExtreme,
    LeastExtreme,
    Max,
    Min,
    Average,
}
impl Distribution<InterferenceMethods> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> InterferenceMethods {
        match rng.gen_range(0..=4) {
            0 => InterferenceMethods::MostExtreme,
            1 => InterferenceMethods::LeastExtreme,
            2 => InterferenceMethods::Max,
            3 => InterferenceMethods::Min,
            _ => InterferenceMethods::Average,
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
pub enum AccelMethods {
    #[default]
    DEFAULT,
    Enabled,
    Disabled,
}
impl Distribution<AccelMethods> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AccelMethods {
        match rng.gen_range(0..=1) {
            0 => AccelMethods::Enabled,
            _ => AccelMethods::Disabled,
        }
    }
}


#[derive(Debug, Default)]
pub struct Accel {
    scale: f64,
    amp: f64,
    pack: super::PackMethods,
    accel: AccelMethods,
}
impl Distribution<Accel> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Accel {
        Accel {
            scale: rng.gen_range(2.0..30.0),
            amp: rng.gen_range(0.0..0.1),
            pack: rand::random(),
            accel: rand::random(),
        }
    }
}

/*
A wave is a curve on a line.
Each wave may have different scaling
and display packing options.
*/
#[derive(Debug, Default)]
pub struct Wave {
    scale: f64,
    pack_method: super::PackMethods,
    accel: Accel,
}
impl Distribution<Wave> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Wave {
        let pack_method: super::PackMethods = rand::random();
        Wave {
            scale: rng.gen_range(2.0..30.0)
                * if let super::PackMethods::ScaleToFit = pack_method {
                    2.0
                } else {
                    1.0
                },
            pack_method,
            accel: rand::random(),
        }
    }
}

/*
A wavepacket is a group of waves on the same line.
A wavepacket has an origin and an angle. All waves
in the packet are calculated relative to that line.
*/
#[derive(Debug, Default)]
pub struct WavePacket {
    origin: super::GeneratorPoint,
    angle: f64,
    wave: Wave,
}
impl Distribution<WavePacket> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> WavePacket {
        WavePacket {
            origin: rand::random(),
            angle: rng.gen_range(0.0..std::f64::consts::PI),
            wave: rand::random(),
        }
    }
}

/*
The FlatwaveRec contains all the information
about a flatwave layer. This equals a list of
wavepackets and a description of the way to
interfere them with each other.
*/
#[derive(Debug)]
pub struct FlatwaveParams {
    interference_method: InterferenceMethods,
    pub packets: Vec<WavePacket>,
}
impl FlatwaveParams {
    const MAX_WAVE_PACKETS: usize = 3;
}
impl Default for FlatwaveParams {
    fn default() -> Self {
        FlatwaveParams {
            packets: (0..1).map(|_| Default::default()).collect(),
            interference_method: Default::default(),
        }
    }
}
impl Distribution<FlatwaveParams> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FlatwaveParams {
        FlatwaveParams {
            interference_method: rand::random(),
            packets: (0..=rng.gen_range(1..=FlatwaveParams::MAX_WAVE_PACKETS))
                .map(|_| rand::random())
                .collect(),
        }
    }
}

pub fn generate(pixel: super::GeneratorPoint, params: &FlatwaveParams) -> f64 {
    /*
    Turn the angle from the origin to this point into a right triangle.
    Compute the legs of this triangle. We will use these legs to determine
    where on the linear wave this point happens to fall.
    */
    let mut out = match params.interference_method {
        InterferenceMethods::Min => 1.0_f64,
        InterferenceMethods::MostExtreme => 0.5_f64,
        _ => 0.0_f64,
    };
    for packet in &params.packets {
        let layer = calc_wave_packet(pixel, packet);
        out = if params.packets.len() > 1 {
            match params.interference_method {
                /*
                Is this value's distance from 0.5 greater than the existing
                value's distance from 0.5?
                */
                InterferenceMethods::MostExtreme => {
                    if (layer - 0.5).abs() > (out - 0.5).abs() {
                        layer
                    } else {
                        out
                    }
                }
                /*
                Is this value closer to the median than the existing value?
                */
                InterferenceMethods::LeastExtreme => {
                    if (layer - 0.5).abs() < (out - 0.5).abs() {
                        layer
                    } else {
                        out
                    }
                }
                //Is this value closer to 1 than the existing value?
                InterferenceMethods::Max => f64::max(layer, out),
                //Is this value closer to zero than the existing one was?
                InterferenceMethods::Min => f64::min(layer, out),
                //Sum all the values up and compute the average at the end.
                InterferenceMethods::Average => out + layer,
                //Beats me what to do with this case. It should never happen.
                _ => layer,
            }
        } else {
            layer
        };
    }
    //If we are in average mode, do the averaging now.
    if let InterferenceMethods::Average = params.interference_method {
        return out / params.packets.len() as f64;
    }
    out
}

fn calc_wave_packet(pixel: super::GeneratorPoint, params: &WavePacket) -> f64 {
    /*
    Calculate the value returned by this wave packet.
    We find the origin of the wave and determine how far away and
    at what angle this point lies from that origin.
    Then we feed the distance & traverse values we get into each
    wave. We combine the results with any of several interference schemes.
    */
    //Re-centre the point on our wave's origin.
    let x = pixel.x - params.origin.x;
    let y = pixel.y - params.origin.y;
    //Now figure the length from the origin to this point.
    let hypotenuse = x.hypot(y);
    //Find the angle of the line from this point to the origin.
    let hypangle = (y / x).atan() + params.angle + if x < 0.0 { std::f64::consts::PI } else { 0.0 };
    //Using the angle and the hypotenuse, we can figure out the individual legs.
    let transverse = hypangle.cos() * hypotenuse;
    let distance = hypangle.sin() * hypotenuse;
    //Our return value, for now, is just the value of our wave.
    calc_wave(distance, transverse, &params.wave)
}

fn calc_wave(distance: f64, transverse: f64, params: &Wave) -> f64 {
    /*
    We have a distance and a transverse value for this wave.
    Use them to calculate the value of the wave at this point.
    Then pack the results to fit in the 0..1 allowed output scale.
    */
    super::packed_cos(
        distance
            + match params.accel.accel {
                AccelMethods::Enabled => {
                    super::packed_cos(transverse, params.accel.scale, &params.accel.pack)
                        * params.accel.amp
                },
                _ => {0.0}
            },
        params.scale,
        &params.pack_method,
    )
}
