use std::error::Error;
use std::result::Result;

pub(super) type StaticResult<T> = Result<T, Box<dyn Error>>;
pub(super) type ScopedResult<'a, T> = Result<T, Box<dyn Error + 'a>>;
