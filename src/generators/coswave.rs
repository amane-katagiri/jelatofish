use rand::Rng;

#[derive(Debug)]
pub enum WaveAccelMethods {
    DEFAULT,
    AccelNone,
    AccelLinear,
    //AccelSine,
}

#[derive(Debug)]
pub struct CoswaveParams {
    origin_h: f64,
    origin_v: f64,
    wave_scale: f64,
    squish: f64,
    sqangle: f64,
    distortion: f64,
    pack_method: super::PackMethods,
    accel_method: WaveAccelMethods,
    accel: f64,
}

pub fn rand_param() -> CoswaveParams {
    let mut rng = rand::thread_rng();
    let mut params = CoswaveParams {
        origin_h: rng.gen_range(0.0..=1.0),
        origin_v: rng.gen_range(0.0..=1.0),
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
        params.accel_method = WaveAccelMethods::AccelLinear;
        params.accel = rng.gen_range(0.0..=2.0) + 1.0;
    } else {
        params.accel_method = WaveAccelMethods::AccelNone;
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

pub fn generate(h: f64, v: f64, params: &CoswaveParams) -> f64 {
    //Rotate the axes of this shape.
    let h = h - params.origin_h;
    let v = v - params.origin_v;

    let hypangle = ((v / h) * params.distortion).atan() + params.sqangle;
    let hypotenuse = h.hypot(v);

    let h = hypangle.cos() * hypotenuse;
    let v = hypangle.sin() * hypotenuse;

    //Calculate the squished distance from the origin to the desired point.
    let hypotenuse = (h * params.squish).hypot(v / params.squish);
    //Scale the wavescale according to our accelerator function.
    let compwavescale = match params.accel_method {
        WaveAccelMethods::AccelNone => params.wave_scale,
        _ => params.wave_scale.powf(hypotenuse * params.accel),
    };
    let rawcos = super::packed_cos(hypotenuse, compwavescale, &params.pack_method);
    (rawcos + 1.0) / 2.0
}
