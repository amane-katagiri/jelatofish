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

use super::super::game;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Debug)]
#[derive(Default)]
struct BoundingBox {
    top_left: super::GeneratorPoint,
    right_bottom: super::GeneratorPoint,
}
impl BoundingBox {
    fn new(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        BoundingBox {
            top_left: super::GeneratorPoint::new(left, top),
            right_bottom: super::GeneratorPoint::new(right, bottom),
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
struct Range {
    min: f64,
    max: f64,
}
impl Range {
    fn new(min: f64, max: f64) -> Self {
        if max > min {
            return Range {
                min: min,
                max: max,
            }
        }
        Range {
            min: max,
            max: min,
        }
    }
}
impl Range {
    fn random(min_range: std::ops::Range<f64>, max_range: std::ops::Range<f64>) -> Range {
        let mut rng = game::get_rng();
        Range::new(rng.gen_range(min_range), rng.gen_range(max_range))
    }
    fn sample(&self) -> f64 {
        if self.min != self.max {
            let mut rng = game::get_rng();
            return rng.gen_range(self.min..self.max);
        }
        self.min
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct Bubble {
    //by what factor should we shrink the influence of this bubble?
    scale: f64,
    //we multiply the h by this and divide the v by it
    squish: f64,
    //how far should we rotate this bubble's coordinate system?
    angle: f64,
    //coordinates for the origin of the bubble
    origin: super::GeneratorPoint,
    //approximate bounding box for the circle
    bound: BoundingBox,
}
impl Bubble {
    fn random(scale: &Range, squish: &Range, angle: &Range) -> Self {
        let scale = scale.sample();
        let origin: super::GeneratorPoint = rand::random();
        Bubble {
            scale: scale,
            squish: squish.sample(),
            angle: angle.sample(),
            origin: origin,
            bound: BoundingBox::new(
                origin.x - scale,
                origin.y - scale,
                origin.x + scale,
                origin.y + scale,
            )
        }
    }
}

#[derive(Debug)]
pub struct BubbleParams {
    scale: Range,
    squish: Range,
    angle: Range,
    bubbles: Vec<Bubble>,
}
impl BubbleParams {
    const MAX_BUBBLES: usize = 32;
    const MIN_BUBBLES: usize = BubbleParams::MAX_BUBBLES / 4;
}
impl Default for BubbleParams {
    fn default() -> Self {
        BubbleParams {
            bubbles: (0..BubbleParams::MIN_BUBBLES).map(|_| {
                Default::default()
            }).collect(), ..Default::default()
        }
    }
}
impl Distribution<BubbleParams> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BubbleParams {
        let scale = Range::random(0.0..0.2, 0.0..0.2);
        let squish = Range::new(
            if game::maybe() {
                let val = rng.gen_range(1.0..4.0);
                if game::maybe() {val} else {1.0/val}
            } else {1.0},
            if game::maybe() {
                let val = rng.gen_range(1.0..4.0);
                if game::maybe() {val} else {1.0/val}
            } else {1.0},
        );
        let angle = Range::random(
            0.0..std::f64::consts::PI / 2.0,
            0.0..std::f64::consts::PI / 2.0,
        );
        let bubbles = (0..rng.gen_range(BubbleParams::MIN_BUBBLES..BubbleParams::MAX_BUBBLES))
            .map(|_| {
                Bubble::random(&scale, &squish, &angle)
            }).collect();
        BubbleParams {
            scale: scale,
            squish: squish,
            angle: angle,
            bubbles: bubbles,
        }
    }
}

pub fn generate(pixel: super::GeneratorPoint, params: &BubbleParams) -> f64 {
    /*
    Calculate nine values from the array of bubbles, corresponding to
    the main tile and each of its neighboring imaginary tiles.
    This lets the edges of the bubbles spill over and affect neighbouring
    tiles, creating the illusion of an infinitely tiled seamless space.
    We damp down the influence of neighbouring tiles proportionate to their
    distance from the edge of the main tile. This is to prevent really huge
    bubbles that cover multiple tiles from breaking the smooth edges.
    */
    [
        get_all_bubbles_value(pixel, params),
        get_all_bubbles_value(
            super::GeneratorPoint::new(pixel.x + 1.0, pixel.y), params
        ) * (1.0 - pixel.x), //right
        get_all_bubbles_value(
            super::GeneratorPoint::new(pixel.x - 1.0, pixel.y), params
        ) * pixel.x, //left
        get_all_bubbles_value(
            super::GeneratorPoint::new(pixel.x, pixel.y + 1.0), params
        ) * (1.0 - pixel.y), //bottom
        get_all_bubbles_value(
            super::GeneratorPoint::new(pixel.x, pixel.y - 1.0), params
        ) * pixel.y, //top
        get_all_bubbles_value(
            super::GeneratorPoint::new(pixel.x + 1.0, pixel.y + 1.0), params
        ) * (1.0 - pixel.x) * (1.0 - pixel.y), //bottom right
        get_all_bubbles_value(
            super::GeneratorPoint::new(pixel.x + 1.0, pixel.y - 1.0), params
        ) * (1.0 - pixel.x) * pixel.y, //top right
        get_all_bubbles_value(
            super::GeneratorPoint::new(pixel.x - 1.0, pixel.y + 1.0), params
        ) * pixel.x * (1.0 - pixel.y), //bottom left
        get_all_bubbles_value(
            super::GeneratorPoint::new(pixel.x - 1.0, pixel.y - 1.0), params
        ) * pixel.x * pixel.y, //bottom right
    ].iter().fold(0.0/0.0, |m, v| v.max(m))
}

fn get_all_bubbles_value(pixel: super::GeneratorPoint, params: &BubbleParams) -> f64 {
    /*
    Get the biggest lump we can from this array of bubbles.
    We just scan through the list, compare the point with each bubble,
    and return the best match we can find.
    */
    params.bubbles.iter().map(|bubble| {
        get_one_bubble_value(pixel, bubble)
    }).fold(0.0/0.0, f64::max)
}

fn get_one_bubble_value(pixel: super::GeneratorPoint, params: &Bubble) -> f64 {
    /*
    Rotate the h and v values around the origin of the bubble according
    to the bubble's angle. Then pass the new h and v on to the squisher.
    */
    //Move the coordinates into bubble-relative coordinates.
    let x = pixel.x - params.origin.x;
    let y = pixel.y - params.origin.y;
    //Calculate the distance from the new origin to this point.
    let hypotenuse = x.hypot(y);
    /*
    Draw a line from the origin to this point. Get the angle this line
    forms with the horizontal. Then add the amount this bubble is rotated.
    */
    let hypangle = (y / x).atan() + params.angle
        //The next line is magic. I don't quite understand it.
        + if x < 0.0 {std::f64::consts::PI} else {0.0};
    //We have the angle and the hypotenuse. Take the sine and cosine to get
    //the new horizontal and vertical distances in the new coordinate system.
    let transverse = hypangle.cos() * hypotenuse + params.origin.x;
    let distance = hypangle.sin() * hypotenuse + params.origin.y;
    //That's it. Pass in the transverse and distance values as the new h and v.
    return get_squished_bubble_value(transverse, distance, params);
}

fn get_squished_bubble_value(transverse: f64, distance: f64, params: &Bubble) -> f64 {
    /*
    Perform the h, v compensation here. We multiply the h by the squish
    value and divide the v by it. So if squish is less than zero, the effect
    is reversed. Very simple little effect that gets non-spherical bubbles.
    */
    let transverse = params.origin.x + ((transverse - params.origin.x) * params.squish);
    let distance = params.origin.y + ((distance - params.origin.y) / params.squish);
    /*
    Calculate the value of this point inside this bubble. If the point
    is outside the bubble, this will return a negative number. If the point
    is on the bubble's radius, this will return zero. Otherwise, this will return
    a number between zero and 1.
    */
    let hypotenuse = (transverse - params.origin.x).hypot(distance - params.origin.y);
    return 1.0 - hypotenuse * hypotenuse / params.scale;
}
