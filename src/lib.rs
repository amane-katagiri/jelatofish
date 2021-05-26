pub mod generators;

pub fn generate(
    h: usize, v: usize, generator: generators::Generators, params: generators::GeneratorParams
) -> Vec<Vec<f64>>{
    vec![vec![0 as f64; v]; h].iter().enumerate().map(
        |(_h, line)| {
            line.iter().enumerate().map(
                |(_v, _)| {
                    let _h= _h as f64 / h as f64;
                    let _v= _v as f64 / h as f64;
                    let _p = match generator {
                        generators::Generators::Coswave
                            => generators::coswave::generate(_h, _v, &params.coswave),
                        generators::Generators::Spinflake
                            => generators::spinflake::generate(_h, _v, &params.spinflake),
                        _ => generators::test::generate(_h, _v),
                    };
                    f64::min(1.0, f64::max(0.0, _p))
                }
            ).collect()
        }
    ).collect()
}
