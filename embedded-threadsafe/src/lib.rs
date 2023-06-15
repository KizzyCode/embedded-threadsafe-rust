#![no_std]
#![doc = include_str!("../README.md")]

mod runtime;

pub mod lazy;
pub mod safecells;

// Re-export the cells
pub use crate::{
    lazy::LazyCell,
    safecells::{interrupt::InterruptCell, local::LocalCell, shared::SharedCell},
};
