#![no_std]
#![doc = include_str!("../README.md")]

mod lazy;
mod runtime;
pub mod singletons;

// Re-export the singletons
pub use crate::singletons::{interrupt::InterruptSingleton, local::LocalSingleton, shared::SharedSingleton};
