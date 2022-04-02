#![forbid(unsafe_code)]
#![deny(clippy::all)]

#[macro_use]
extern crate lazy_static;

mod swiss_lotto;
mod errors;

#[doc(hidden)]
pub use swiss_lotto::*;

#[doc(hidden)]
pub use errors::*;

