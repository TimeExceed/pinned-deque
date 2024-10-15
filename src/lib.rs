#![doc = include_str!("../README.md")]

mod r#impl;
pub use self::r#impl::*;

mod cmp;
mod intf;
mod iter;
pub use self::iter::*;
mod chunk;

#[cfg(test)]
mod tests;
