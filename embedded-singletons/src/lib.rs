#![no_std]
#![doc = include_str!("../README.md")]

mod runtime;
pub mod singleton;

// Re-export the singletons
pub use crate::singleton::*;
