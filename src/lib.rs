pub mod generators;

use std::path::Path;
use image;
use rand::Rng;

const MAX_CHANVAL: f64 = 255.0;

pub fn save_image(h: usize, v: usize, filename: &str) {
    let params = generators::GeneratorParams {
        generator: rand::random(),
        coswave: generators::coswave::rand_param(),
        spinflake: generators::spinflake::rand_param(),
    };

    let fish = make_jelatofish(h, v, &vec![], &params);
    let mut imgbuf = image::ImageBuffer::new(h as u32, v as u32);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let p = get_jelatofish_pixel(x as usize, y as usize, &fish);
        *pixel = image::Rgb([
            (p.r * MAX_CHANVAL) as u8,
            (p.b * MAX_CHANVAL) as u8,
            (p.g * MAX_CHANVAL) as u8,
        ]);
    }
    imgbuf.save(&Path::new(filename)).unwrap();
}

const MAX_LAYERS: usize = 6;
const MIN_LAYERS: usize = 2;

#[derive(Debug)]
pub struct Colour {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

#[derive(Debug)]
pub struct ColourLayer {
    //The image layer, a reference to pixels.
    image: Vec<Vec<f64>>,
    //The foreground colour, used for high image values.
    fore: Colour,
    //The background colour, used for low image values.
    back: Colour,
    //The mask image. If filled with 0, we use the image layer as its own mask.
    mask: Option<Vec<Vec<f64>>>,
    //If the flag is true, we invert the mask.
    invert_mask: bool,
}

#[derive(Debug)]
pub struct Jelatofish {
    h: usize,
    v: usize,
    cutoff_threshold: f64,
    layers: Vec<ColourLayer>,
}

pub fn make_jelatofish(
    h: usize, v: usize, colours: &Vec<Colour>, params: &generators::GeneratorParams
) -> Jelatofish {
    let mut rng = rand::thread_rng();

    /*
    Create a series of layers which we will later use to generate
    pixel data. These will contain the complete package of settings
    used to calculate image values.
    */
    /*
    Now allocate random layers to use for the image and mask of this layer.
    Half the time, we use the image as its own mask.
    Half the time, we invert the mask.
    */
    Jelatofish {
        h: h,
        v: v,
        cutoff_threshold: rng.gen_range(0.0..=1.0 / 16.0),
        layers: vec![0; rng.gen_range(MIN_LAYERS..=MAX_LAYERS)].iter().map(|_| {
            let back = random_palette_pixel(colours);
            let fore = loop {
                let fore = random_palette_pixel(colours);
                if fore.r != back.r || fore.g != back.g || fore.b != back.b {
                    break fore;
                }
            };
            ColourLayer {
                image: generators::generate(h, v, &params),
                //Flip a coin. If it lands heads-up, create another layer for use as a mask.
                mask: if rng.gen_range(0..2) == 0 {Some(generators::generate(h, v, &params))} else {None},
                //Flip another coin. If it lands heads-up, set the flag so we invert this layer.
                invert_mask: rng.gen_range(0..2) == 0,
                back: back,
                fore: fore,
            }
        }).collect()
    }
}

fn random_palette_pixel(colours: &Vec<Colour>) -> Colour {
    /*
    Pick a random pixel from this palette.
    If the palette is empty, create it from random values.
    */
    let mut rng = rand::thread_rng();

    if colours.len() > 1 {
        let c = &colours[rng.gen_range(0..colours.len())];
        Colour {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    } else {
        Colour {
            r: rng.gen_range(0.0..=1.0),
            g: rng.gen_range(0.0..=1.0),
            b: rng.gen_range(0.0..=1.0),
            a: 0.0,
        }
    }
}

pub fn get_jelatofish_pixel(x: usize, y: usize, fish: &Jelatofish) -> Colour {
    /*
    Calculate one pixel.
    We start with a black pixel.
    Then we loop through all of the layers, calculating each one with its
    mask. We then merge each layer's resulting pixel onto the out image.
    Once we're done, we return the merged pixel.
    We use alpha kind of backwards: high values mean high opacity, low values
    mean low opacity.
    */
    //Did we get valid parameters?
    if x >= fish.h && y >= fish.v {
        return Colour {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }
    let mut outval = Colour {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    for layer in &fish.layers {
        //Get the image value for this pixel, for this layer.
        let imageval = layer.image[x][y];
        let maskval = match &layer.mask {
            Some(mask) => mask[x][y],
            None => layer.image[x][y],
        };
        let maskval = if layer.invert_mask {1.0 - maskval} else {maskval};
        /*
        Now we are ready. Calculate the image value for this layer.
        We use the image value as the proportion of the distance between
        two colours. We calculate this one channel at a time. This results
        in a smooth gradient of colour from min to max.
        */
        let mut layerpixel = Colour {
            r: imageval * (layer.fore.r - layer.back.r) + layer.back.r,
            g: imageval * (layer.fore.g - layer.back.g) + layer.back.g,
            b: imageval * (layer.fore.b - layer.back.b) + layer.back.b,
            a: maskval,
        };
        /*
        The image value for this layer is calculated.
        But the image is more than just this layer: it is the merged
        results of all the layers. So now we merge this value with
        the existing value calculated from the previous layers.
        We use the alpha channel to determine the proportion of blending.
        The new layer goes behind the existing layers; we use the existing
        alpha channel to determine what proportion of the new value shows
        through.
        */
        outval.r = (outval.r * outval.a) + (layerpixel.r * (1.0 - outval.a));
        outval.g = (outval.g * outval.a) + (layerpixel.g * (1.0 - outval.a));
        outval.b = (outval.b * outval.a) + (layerpixel.b * (1.0 - outval.a));
        /*
        Add the alpha channels (representing opacity); if the result is greater
        than 100% opacity, we just stop calculating (since no further layers
        will produce visible data).
        */
        layerpixel.a = layerpixel.a * (1.0 - outval.a);
        if layerpixel.a + outval.a + fish.cutoff_threshold >= 1.0 {
            outval.a = 1.0;
            /*
            And now end the loop, because we've collected all the data we need.
            Calculating pixels from any of the deeper layers would just be a waste of time.
            */
            break;
        } else {
            outval.a += layerpixel.a;
        }
    }
    outval
}
