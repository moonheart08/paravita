#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(allocator_api)]
#![feature(variant_count)]
#![feature(ip_in_core)]
#![feature(error_in_core)]
#![no_std]

extern crate alloc;
#[cfg(std)]
extern crate std;

pub mod vm;
