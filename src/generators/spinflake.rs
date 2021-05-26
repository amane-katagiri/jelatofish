use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const MAX_TWIRL: f64 = 14.0;
const MAX_SINEAMP: f64 = 4.0;
const MAX_FLORETS: usize = 3;

#[derive(Debug)]
pub enum SinePositivizingMethods {
    DEFAULT,
    SineCompressMethod,
    SineTruncateMethod,
    SineAbsoluteMethod,
    SineSawbladeMethod,
}
impl Distribution<SinePositivizingMethods> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SinePositivizingMethods {
        match rng.gen_range(0..=3) {
            0 => SinePositivizingMethods::SineCompressMethod,
            1 => SinePositivizingMethods::SineTruncateMethod,
            2 => SinePositivizingMethods::SineAbsoluteMethod,
            _ => SinePositivizingMethods::SineSawbladeMethod,
        }
    }
}

#[derive(Debug)]
pub enum TwirlMethods {
    DEFAULT,
    TwirlNoneMethod,
    TwirlCurveMethod,
    TwirlSineMethod,
    // TwirlAccelMethod,
}
impl Distribution<TwirlMethods> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TwirlMethods {
        match rng.gen_range(0..=2) {
            0 => TwirlMethods::TwirlNoneMethod,
            1 => TwirlMethods::TwirlCurveMethod,
            _ => TwirlMethods::TwirlSineMethod,
        }
    }
}

#[derive(Debug)]
pub struct Floret {
    sinepos_method: SinePositivizingMethods,
    backward: bool,
    spines: i32,
    spine_radius: f64,
    twirl_base: f64,
    twirl_speed: f64,
    twirl_amp: f64,
    twirl_mod: f64,
    twirl_method: TwirlMethods,
}

#[derive(Debug)]
pub struct SpinflakeParams {
    origin_h: f64,
    origin_v: f64,
    radius: f64,
    squish: f64,
    twist: f64,
    average_florets: bool,
    florets: i32,
    layer: Vec<Floret>,
}

pub fn rand_param() -> SpinflakeParams {
    let mut rng = rand::thread_rng();

    let florets = rng.gen_range(0..=(MAX_FLORETS as i32)) + 1;
    let params = SpinflakeParams {
        origin_h: rng.gen_range(0.0..=1.0),
        origin_v: rng.gen_range(0.0..=1.0),
        radius: rng.gen_range(0.0..=1.0),
        squish: rng.gen_range(0.0..=2.75) * 0.25,
        twist: rng.gen_range(0.0..=std::f64::consts::PI),
        average_florets: rng.gen_range(0..2) == 0,
        florets: florets,
        layer: (0..florets).map(|_| {
            let mut floret = Floret{
                sinepos_method: rand::random(),
                backward: rng.gen_range(0..2) == 0,
                spines: rng.gen_range(0..=15) + 1,
                spine_radius: rng.gen_range(0.0..=0.5),
                twirl_base: rng.gen_range(0.0..=std::f64::consts::PI),
                twirl_method: rand::random(),
                twirl_speed: 0.0,
                twirl_amp: 0.0,
                twirl_mod: 0.0,
            };
            if let SinePositivizingMethods::SineAbsoluteMethod = floret.sinepos_method {
                if floret.spines % 2 == 1 {
                    floret.spines += 1;
                }
            }
            match floret.twirl_method {
                TwirlMethods::TwirlSineMethod => {
                    floret.twirl_speed = rng.gen_range(0.0..=(MAX_TWIRL * std::f64::consts::PI));
                    floret.twirl_amp = rng.gen_range(-MAX_SINEAMP..=MAX_SINEAMP);
                    floret.twirl_mod = rng.gen_range(-0.5..=0.5);
                },
                TwirlMethods::TwirlCurveMethod => {
                    floret.twirl_speed = rng.gen_range(-MAX_TWIRL..=MAX_TWIRL);
                    floret.twirl_amp = rng.gen_range(-MAX_SINEAMP..=MAX_SINEAMP);
                },
                _ => {},
            }
            floret
        }).collect(),
    };

    params
}

pub fn generate(h: f64, v: f64, params: &SpinflakeParams) -> f64 {
    let point = vtiledpoint(h, v, params);
    if h > 0.5 {
        let farpoint = vtiledpoint(h - 1.0, v, params);
        let farweight = (h - 0.5) * 2.0;
        let weight = 1.0 - farweight;
        return (point * weight) + (farpoint * farweight);
    }
    point
}

fn chopsin(theta: f64, params: &Floret) -> f64
    {
    let out = theta.sin();
    let out = match params.sinepos_method {
        SinePositivizingMethods::SineCompressMethod =>(out + 1.0) / 2.0,
        SinePositivizingMethods::SineAbsoluteMethod => out.abs(),
        SinePositivizingMethods::SineTruncateMethod => if out < 0.0 {out + 1.0} else {out},
        SinePositivizingMethods::SineSawbladeMethod => {
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

fn vtiledpoint(h: f64, v: f64, params: &SpinflakeParams) -> f64 {
    let point = rawpoint(h, v, params);
    if v > 0.5 {
        let farpoint = rawpoint(h, v - 1.0, params);
        let farweight = (v - 0.5) * 2.0;
        let weight = 1.0 - farweight;
        return (point * weight) + (farpoint * farweight);
    }
    point
}

fn rawpoint(h: f64, v: f64, params: &SpinflakeParams) -> f64 {
    /*
    Rotate the point around our origin. This lets the squashed bulge-points on
    the sides of the squished spinflake point in random directions - not just aligned
    with the cartesian axes.
    */
    let h = h - params.origin_h;
    let v = v - params.origin_v;

    let hypangle = (v / h).atan() + params.twist;
    let origindist = h.hypot(v);

    let h = hypangle.cos() * origindist;
    let v = hypangle.sin() * origindist;
    //Calculate the distance from the origin to this point. Again.
    let origindist = (h * params.squish).hypot(v / params.squish);
    //If we are at the origin, there is no need to do the computations.
    if origindist != 0.0 {
        //The edge is (currently) a circle some radius units away.
        //Compute the angle this point represents to the origin.
        let pointangle = (v / h).atan();
        let mut edgedist = params.radius;
        for layer in &params.layer {
            edgedist += calcwave(pointangle, origindist, &layer);
        }
        let edgedist =
            if params.average_florets {edgedist / (params.florets as f64)} else {edgedist};
        //Our return value is the distance from the edge, proportionate
        //to the distance from the origin to the edge.
        let proportiondist = ((edgedist - origindist) / edgedist) as f64;
        //If the value is >=0, we are inside the shape. Otherwise, we're outside it.
        if proportiondist >= 0.0 {
            return proportiondist.sqrt();
        } else {
            return 1.0 - (1.0 / (1.0 - proportiondist));
        }
    }
    1.0
}

fn calcwave(theta: f64, dist: f64, params: &Floret) -> f64
    {
    /*
    Calculate the distance from centre this floret adds to the mix
    at the particular angle supplied.
    This is where we incorporate the floret's spines and twirling.
    Oddly, a spinflake's florets don't have to twirl in unison. This
    can get really interesting. If it doesn't work, migrate the twirl back
    to the spinflake instead.
    */
    let cosparam = match params.twirl_method {
        TwirlMethods::TwirlCurveMethod => theta * (params.spines as f64) + params.twirl_base
            + (dist * (params.twirl_speed + (dist * params.twirl_amp))),
        TwirlMethods::TwirlSineMethod => (theta * (params.spines as f64) + params.twirl_base)
            + ((dist * params.twirl_speed).sin() * (params.twirl_amp + (dist * params.twirl_amp))),
        _ => theta * (params.spines as f64) + params.twirl_base,
    };
    chopsin(cosparam, params) * params.spine_radius
}
