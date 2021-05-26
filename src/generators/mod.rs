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
