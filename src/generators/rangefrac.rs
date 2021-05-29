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
#[derive(Clone)]
#[derive(Copy)]
struct Point {
    pub x: i32,
    pub y: i32,
}
impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point {
            x: x,
            y: y,
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
struct BoundingBox {
    top_left: Point,
    bottom_right: Point,
}
impl BoundingBox {
    fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        BoundingBox {
            top_left: Point::new(left, top),
            bottom_right: Point::new(right, bottom),
        }
    }
}

#[derive(Debug)]
pub struct RangefracParams {
    data: [[f64; RangefracParams::VALMATRIX_SIZE]; RangefracParams::VALMATRIX_SIZE],
}
impl RangefracParams {
    const VALMATRIX_SCALE: u32 = 8;
    const VALMATRIX_SIZE: usize = 1 << RangefracParams::VALMATRIX_SCALE;
}
impl Default for RangefracParams {
    fn default() -> Self {
        RangefracParams {
            data: [[0.0; RangefracParams::VALMATRIX_SIZE]; RangefracParams::VALMATRIX_SIZE],
        }
    }
}
impl Distribution<RangefracParams> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RangefracParams {
        /*
        Walk through the matrix.
        For each point, search its neighbors. For each neighboring point
        of higher level than current, compare its value against the current
        min and max. If the neighboring point exceeds min or max, use its
        value as the new min or max. Repeat.
        */
        let mut level = [[0 as i32; RangefracParams::VALMATRIX_SIZE]; RangefracParams::VALMATRIX_SIZE];
        let mut data = [[0.0; RangefracParams::VALMATRIX_SIZE]; RangefracParams::VALMATRIX_SIZE];

        for step in 1..=RangefracParams::VALMATRIX_SCALE {
            let step = (2 as usize).pow(RangefracParams::VALMATRIX_SCALE - step);
            for x in (0..RangefracParams::VALMATRIX_SIZE).step_by(step) {
                for y in (0..RangefracParams::VALMATRIX_SIZE).step_by(step) {
                    let step = step as i32;
                    //See if we need to calculate this pixel at all.
                    if level[x][y] < step {
                        //Go hunting for the highest and lowest values among this pixel's neighbors.
                        let xi = x as i32;
                        let yi = y as i32;
                        let local_values: Vec<f64> = [
                            //Top left
                            (xi - step, yi - step),
                            //Top
                            (xi, yi - step),
                            //Top right
                            (xi + step, yi - step),
                            //Left
                            (xi - step, yi),
                            //Right
                            (xi + step, yi),
                            //Bottom left
                            (xi - step, yi + step),
                            //Bottom
                            (xi, yi + step),
                            //Bottom right
                            (xi + step, yi + step),
                        ].iter().filter(|p| level[wrap_x(p.0)][wrap_y(p.1)] > step)
                            .map(|p| data[wrap_x(p.0)][wrap_y(p.1)]).collect();
                        let max = if local_values.len() > 0 {
                            local_values.iter().fold(0.0/0.0, |m, v| v.max(m))
                        } else {0.0};
                        let min = if local_values.len() > 0 {
                            local_values.iter().fold(0.0/0.0, |m, v| v.min(m))
                        } else {1.0};
                        let val = if min != max {
                            if min > max {
                                rng.gen_range(max..min)
                            } else {
                                rng.gen_range(min..max)
                            }
                        } else {min};
                        /*
                        The first pieces of data are always picked completely at random,
                        because they have no neighbors to influence their decisions.
                        But these data are the extremes of the image - no values can be
                        any larger or smaller than them. So we "push" them out a little
                        bit by rounding them to integer values, then averaging them with
                        their original values. This gives us whiter whites and blacker
                        blacks, without forcing the first data to be pure white or black.
                        */
                        let val = if step >= RangefracParams::VALMATRIX_SIZE as i32 / 2 {
                            (val + if val > 0.5 {1.0} else {0.0}) / 2.0
                        } else {val};
                        data[x][y] = val;
                        level[x][y] = step;
                    }
                }
            }
        }
        RangefracParams {
            data: data
        }
    }
}

#[derive(Debug)]
struct LocalParam {
    value: f64,
    weight: f64,
}

pub fn generate(pixel: super::GeneratorPoint, params: &RangefracParams) -> f64 {
    /*
    Locate the closest values to this one in the value
    array. Then use a proportional average based on distance
    to get the returned value.
    */
    /*
    Get each known value near the one we have been requested to retrieve.
    Calculate the distance from the requested point to each known point.
    Use the distance as a weight in an average.
    This essentially scales a small pixel map into a large one, using linear
    interpolation. It could be generalized with a little work.
    */
    let tweaker = 0.5 / RangefracParams::VALMATRIX_SIZE as f64;
    let left = f64::floor(pixel.x * RangefracParams::VALMATRIX_SIZE as f64 - tweaker) as i32;
    let top = f64::floor(pixel.y * RangefracParams::VALMATRIX_SIZE as f64 - tweaker) as i32;
    let bound = BoundingBox::new(left, top, left + 1, top + 1);
    let local_params: Vec<LocalParam> = [
        //TOPLEFT
        (bound.top_left.x, bound.top_left.y),
        //TOPRIGHT
        (bound.bottom_right.x, bound.top_left.y),
        //BOTLEFT
        (bound.top_left.x, bound.bottom_right.y),
        //BOTRIGHT
        (bound.bottom_right.x, bound.bottom_right.y),
    ].iter().map(|p| LocalParam {
        value: params.data[wrap_x(p.0)][wrap_y(p.1)],
        weight: calc_weight(p.0, p.1, pixel)
    }).collect();
    let total_sum = local_params.iter().map(|v| v.value * v.weight).fold(0.0, |sum, x| sum + x);
    let total_weight = local_params.iter().map(|v| v.weight).fold(0.0, |sum, x| sum + x);
    total_sum / total_weight
}

fn calc_weight(matrix_width: i32, matrix_height: i32, pixel: super::GeneratorPoint) -> f64 {
    f64::max(
        0.0,
        1.0 - (matrix_width as f64 - (pixel.x * RangefracParams::VALMATRIX_SIZE as f64))
            .hypot(matrix_height as f64 - (pixel.y * RangefracParams::VALMATRIX_SIZE as f64))
    )
}

fn wrap_x(coord: i32) -> usize {
    wrap(coord)
}
fn wrap_y(coord: i32) -> usize {
    wrap(coord)
}
fn wrap(coord: i32) -> usize {
    match coord {
        x if x >= 0 => (x as usize) % RangefracParams::VALMATRIX_SIZE,
        x => (
            x % RangefracParams::VALMATRIX_SIZE as i32 + RangefracParams::VALMATRIX_SIZE as i32
        ) as usize,
    }
}
