mod impls;

use impls::*;

pub fn tododo() -> usize {
    hello!(1, 2)
}

pub fn tododo2() -> usize {
    hello!(1, 2, c = 4)
}

pub fn tododo3<const N: usize>() -> usize {
    hello_with_generics!(<N>, 1, 2, c = 4)
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

#[test]
fn test_call_generic() {
    let partial = hello_with_generics!(<5>, 1, 2, c = 4);
    let full = hello_with_generics_impl::<5>(1, 2, 4);
    assert_eq!(full, partial);
}

// #[test]
// fn test_call_generic_with_fn_like_default_value() {
//     let partial = hello_with_generics_and_fn_like_expression!(<5>, 1, 2);
//     let full = hello_with_generics_impl::<5>(1, 2, 1*2*1*2);
//     assert_eq!(full, partial);
// }