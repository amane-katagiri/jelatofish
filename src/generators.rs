use libm;

fn coswave() {}
fn spinflake() {}
fn rangefrac() {}
fn flatwave() {}
fn bubble() {}

pub fn test(h: f64, v: f64) -> f64 {
    libm::exp(-h * v)
}
