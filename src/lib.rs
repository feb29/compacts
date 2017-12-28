#![feature(conservative_impl_trait)]
#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]
// #![deny(warnings)]

extern crate byteorder;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;

pub mod bits;
pub use bits::BitSet;
