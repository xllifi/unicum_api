use std::{
    fmt::{Debug, Display},
    str::{Chars, FromStr},
};

use log::trace;

use crate::{entities, impls::unicum_api::ModuleError};

/// A trait to temporarily view a type as a string slice without allocating.
pub trait AsStrRef {
    fn with_str<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&str) -> R;
}

// 1. Base implementation for `str`
impl AsStrRef for &str {
    fn with_str<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        f(self)
    }
}

// 2. Base implementation for `char` (uses a small 4-byte stack buffer)
impl AsStrRef for char {
    fn with_str<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        let mut buf = [0; 4];
        f(self.encode_utf8(&mut buf))
    }
}

pub fn parse_err<S: ToString>(string: S) -> ModuleError {
    ModuleError::ParseError {
        cause: string.to_string(),
    }
}
pub fn parse_next<'a, T, U, Iter>(split: &mut Iter) -> Result<T, ModuleError>
where
    T: FromStr,
    T::Err: Display,
    U: AsStrRef + Debug,
    Iter: Iterator<Item = U>,
{
    let next = split.next();
    trace!("Trying to convert {next:?}");

    next.ok_or(parse_err(""))?
        .with_str(|s| s.trim().parse::<T>())
        .map_err(|e| parse_err(format!("{e}")))
}

pub fn to_digit_next(chars: &mut Chars, radix: u32) -> Result<u32, ModuleError> {
    let next = chars.next();
    trace!("Trying to convert {next:?}");

    next.ok_or(parse_err(""))?
        .to_digit(radix)
        .ok_or(parse_err(format!("Failed to parse {next:?} as number")))
}

impl From<time::Date> for entities::Date {
    fn from(value: time::Date) -> Self {
        Self {
            day: value.day(),
            month: value.month() as u8,
            year: value.year(),
        }
    }
}

pub fn sel_to_row_and_col<S: AsRef<str>>(sel: S) -> Result<(u8, u8), ModuleError> {
    let mut chars = sel.as_ref().chars();
    let row: u8 = to_digit_next(&mut chars, 16)? as u8;
    let col: u8 = to_digit_next(&mut chars, 16)? as u8;

    Ok((row, col))
}
