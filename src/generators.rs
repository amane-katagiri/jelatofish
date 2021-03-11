use libm;

pub fn coswave(h: f64, v: f64) -> f64 {
    let hypangle = libm::atan((v / h) * 0.5) + 0.6;
    let hypotenuse = libm::hypot(h, v);
    let h = libm::cos(hypangle) * hypotenuse;
    let v = libm::sin(hypangle) * hypotenuse;
    let hypotenuse = libm::hypot(h * 0.7, v / 0.7);
    let compwavescale = libm::pow(1.8, hypotenuse * 1.9);
    let rawcos = libm::cos(hypotenuse * compwavescale);
    (rawcos + 1.0) / 2.0
}
fn spinflake() {}
fn rangefrac() {}
fn flatwave() {}
fn bubble() {}

pub fn test(h: f64, v: f64) -> f64 {
    libm::exp(-h * v)
}
