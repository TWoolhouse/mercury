use crate::{ops::FnOp, Error};

use super::{Arg, Result};

fn banding<const SB: usize, const SR: usize>(bands: [f64; SB], rates: [f64; SR]) -> FnOp<f64> {
    Box::new(move |account| {
        let mut acc = 0.0;
        for (bounds, rate) in bands.windows(2).zip(rates) {
            let lower = bounds[0];
            let upper = bounds[1];
            acc += (upper.min(account) - lower).max(0.0) * rate;
        }
        acc
    })
}

pub fn ni(args: &[Arg], names: &'static [&'static str]) -> Result {
    static CLASSES: [(&str, u8); 12] = [
        ("A", 0u8),
        ("B", 1u8),
        ("C", 2u8),
        ("F", 0u8),
        ("H", 0u8),
        ("I", 1u8),
        ("J", 3u8),
        ("L", 3u8),
        ("M", 0u8),
        ("S", 2u8),
        ("V", 0u8),
        ("Z", 3u8),
    ];
    static CLASS_TABLE: [(f64, f64); 4] = [(0.12, 0.02), (0.0585, 0.02), (0.0, 0.0), (0.02, 0.02)];
    let rates = args
        .get(0)
        .ok_or_else(|| Error::ArgumentMissing(names[0]))
        .and_then(|arg| {
            if let Arg::String(class) = arg {
                if let Some((_, idx)) = CLASSES.iter().find(|(cls, _)| cls == class) {
                    Ok(CLASS_TABLE[*idx as usize])
                } else {
                    Err(Error::ArgumentInvalid(class.clone()))
                }
            } else {
                Err(Error::ArgumentInvalidType(names[0]))
            }
        })?;

    let rates = [0.0, rates.0, rates.1];
    let bands = [0.0, 242.0, 967.0, f64::MAX];

    Ok(("National Insurance".to_owned(), banding(bands, rates)))
}

pub fn inc(_args: &[Arg], _names: &'static [&'static str]) -> Result {
    let rates = [0.0, 0.2, 0.4, 0.45];
    let bands = [0.0, 12_570.0, 50_270.0, 125_140.0, f64::MAX];
    Ok(("Income Tax".to_owned(), banding(bands, rates)))
}
pub fn sfe(args: &[Arg], names: &'static [&'static str]) -> Result {
    static PLANS: [(u8, f64); 4] = [(1, 22_015.0), (2, 27_295.0), (4, 27_660.0), (5, 25_000.0)];
    let band = args
        .get(0)
        .ok_or_else(|| Error::ArgumentMissing(names[0]))
        .and_then(|arg| {
            if let Arg::Number(plan_f) = arg {
                let plan = *plan_f as u8;
                if let Some((_, band)) = PLANS.iter().find(|(pln, _)| *pln == plan) {
                    Ok(band)
                } else {
                    Err(Error::ArgumentInvalid(plan_f.to_string()))
                }
            } else {
                Err(Error::ArgumentInvalidType(names[0]))
            }
        })?;

    let rates = [0.0, 0.09];
    let bands = [0.0, *band, f64::MAX];

    Ok(("Student Finance".to_owned(), banding(bands, rates)))
}
