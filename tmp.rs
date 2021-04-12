#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
mod impls {
    use default_params_derive::*;
    pub fn hello_impl(a: usize, b: usize, c: usize) -> usize {
        a * b + c
    }
    pub fn hello_with_generics_impl<const N: usize>(a: usize, b: usize, c: usize) -> usize {
        a * b + c + N
    }
}
use impls::*;
pub fn tododo() -> usize {
    hello_impl(1, 2, 3usize)
}
pub fn tododo2() -> usize {
    hello_impl(1, 2, 4)
}
pub fn tododo3<const N: usize>() -> usize {
    hello_with_generics_impl::<N>(1, 2, 4)
}
