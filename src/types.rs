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

// must be 0.0..=1.0
pub type PixelVal = f64;

pub type PixelMap = Vec<Vec<f64>>;

#[derive(Debug)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Copy)]
pub struct Area {
    pub width: usize,
    pub height: usize,
}
impl Area {
    pub fn new(width: usize, height: usize) -> Self {
        Area {
            width: width,
            height: height,
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Copy)]
pub struct PixelPoint {
    pub x: usize,
    pub y: usize,
}
impl PixelPoint {
    pub fn new(x: usize, y: usize) -> Self {
        PixelPoint {
            x: x,
            y: y,
        }
    }
}
