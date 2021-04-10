use default_params_derive::*;

#[default_params]
pub fn hello(
    a: usize, 
    b: usize, 
    #[default_value(3usize)] c: usize
) -> usize{
    a*b + c
}