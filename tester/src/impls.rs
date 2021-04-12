use default_params_derive::*;

#[default_params]
pub fn hello(
    a: usize, 
    b: usize, 
    #[default_value(3usize)] c: usize
) -> usize{
    a*b + c
}

#[default_params]
pub fn hello_with_generics<const N: usize>(
    a: usize, 
    b: usize, 
    #[default_value(3usize)] c: usize
) -> usize{
    a*b + c + N
}

// #[default_params]
// pub fn hello_with_generics_and_fn_like_expression<const N: usize>(
//     a: usize, 
//     b: usize, 
//     #[default_value(a*b*a*b)] c: usize
// ) -> usize{
//     a*b + c + N
// }