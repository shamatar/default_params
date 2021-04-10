mod impls;

use impls::*;

pub fn tododo() -> usize {
    hello!(1, 2)
}

pub fn tododo2() -> usize {
    hello!(1, 2, c = 4)
}

#[test]
fn test() {
    hello_impl(2, 3, 4);
}

#[test]
fn test_call() {
    let full = hello_impl(1, 2, 3);
    let partial = hello!(1, 2);
    assert_eq!(full, partial);
}

#[test]
fn test_call2() {
    let full = hello_impl(1, 2, 4);
    let partial = hello!(1, 2, c = 4);
    assert_eq!(full, partial);
}