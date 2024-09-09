#![doc = include_str!("../README.md")]

mod r#impl;
pub use self::r#impl::*;

mod cmp;
mod intf;
mod iter;
pub use self::iter::*;
mod page;
use self::page::*;

#[cfg(test)]
mod tests;
