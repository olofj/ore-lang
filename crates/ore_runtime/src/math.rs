use crate::*;

#[no_mangle]
pub extern "C" fn ore_math_sqrt(x: f64) -> f64 { x.sqrt() }
#[no_mangle]
pub extern "C" fn ore_math_sin(x: f64) -> f64 { x.sin() }
#[no_mangle]
pub extern "C" fn ore_math_cos(x: f64) -> f64 { x.cos() }
#[no_mangle]
pub extern "C" fn ore_math_tan(x: f64) -> f64 { x.tan() }
#[no_mangle]
pub extern "C" fn ore_math_log(x: f64) -> f64 { x.ln() }
#[no_mangle]
pub extern "C" fn ore_math_log10(x: f64) -> f64 { x.log10() }
#[no_mangle]
pub extern "C" fn ore_math_exp(x: f64) -> f64 { x.exp() }
#[no_mangle]
pub extern "C" fn ore_math_pow(base: f64, exp: f64) -> f64 { base.powf(exp) }
#[no_mangle]
pub extern "C" fn ore_math_abs(x: f64) -> f64 { x.abs() }
#[no_mangle]
pub extern "C" fn ore_math_floor(x: f64) -> f64 { x.floor() }
#[no_mangle]
pub extern "C" fn ore_math_ceil(x: f64) -> f64 { x.ceil() }
#[no_mangle]
pub extern "C" fn ore_math_round(x: f64) -> f64 { x.round() }
#[no_mangle]
pub extern "C" fn ore_math_pi() -> f64 { std::f64::consts::PI }
#[no_mangle]
pub extern "C" fn ore_math_e() -> f64 { std::f64::consts::E }
#[no_mangle]
pub extern "C" fn ore_math_atan2(y: f64, x: f64) -> f64 { y.atan2(x) }

#[no_mangle]
pub extern "C" fn ore_float_round_to(x: f64, decimals: i64) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (x * factor).round() / factor
}

#[no_mangle]
pub extern "C" fn ore_float_format(x: f64, decimals: i64) -> *mut OreStr {
    let s = format!("{:.prec$}", x, prec = decimals as usize);
    str_to_ore(s)
}

// ── Int math ──

#[no_mangle]
pub extern "C" fn ore_int_pow(base: i64, exp: i64) -> i64 {
    if exp < 0 {
        return 0; // integer pow with negative exponent → 0
    }
    (base as i128).pow(exp as u32) as i64
}
