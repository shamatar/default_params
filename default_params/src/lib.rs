#[allow(missing_fragment_specifier)]

extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{spanned::Spanned, FnArg, Ident, Attribute, ItemFn};
use quote::{quote, TokenStreamExt};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Span};

mod default_value;

// #[proc_macro_attribute]
// #[proc_macro_error]
// pub fn default_value(args: TokenStream, input: TokenStream) -> TokenStream {
//     let mut args = TokenStream2::from(args).into_iter();
//     let mut input = TokenStream2::from(input).into_iter();

//     eprintln!("args = {:?}", args);
//     eprintln!("default value input = {:?}", input);

//     quote!().into()
// }

const DEFAULT_VALUE_ATTR_NAME: &'static str = "default_value";
const DEFAULT_VALUE_FUNCTION_ATTR_NAME: &'static str = "default_value_fn";

use proc_macro_error::{
    proc_macro_error,
    abort,
    abort_call_site,
};

// enum FillType {
//     NamedArgument(TokenStream2),
//     Expression(TokenStream2),
//     Function(TokenStream2),
// }

enum FillTypeAttr {
    Expression(Attribute),
    Function(Attribute),
}


#[proc_macro_attribute]
#[proc_macro_error]
pub fn default_params(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    if !args.is_empty() {
        abort_call_site!(
            "`default_params` macro does not take any attributes"
        );
    }
    
    let parsed_fn = syn::parse::<ItemFn>(input);
    let mut parsed_fn = match parsed_fn {
        Ok(parsed_fn) => parsed_fn,
        Err(err) => {
            abort_call_site!(
                "`default_params` macro failed to parse function signature and body: {}", err
            );
        }
    };

    // check python convensions (for now) to see if all default arguments are after the non-default onces
    let mut first_default_found = false;
    let mut defaultable_arguments = vec![];
    let mut non_default_arguments = vec![];
    for input in parsed_fn.sig.inputs.iter_mut() {
        let input_clone = input.clone();
        match input {
            FnArg::Receiver(_) => {
                non_default_arguments.push(input_clone);
                continue
            },
            FnArg::Typed(ref mut ty) => {
                // abort!(r.span(), "Do not yet support by-reference inputs");
                if ty.attrs.is_empty() {
                    if first_default_found {
                        abort!(ty.span(), "Non-default attribute after a default one");
                    } else {
                        non_default_arguments.push(input_clone);
                        continue
                    }
                } else {
                    let mut specific = None;
                    for (idx, attr) in ty.attrs.iter().enumerate() {
                        if attr.path.is_ident(DEFAULT_VALUE_ATTR_NAME) {
                            if specific.is_none() {
                                specific = Some((idx, input_clone.clone(), FillTypeAttr::Expression(attr.clone())));
                            } else {
                                abort!(attr.span(), "Duplicate attribute for default argument");
                            }
                        } else if attr.path.is_ident(DEFAULT_VALUE_FUNCTION_ATTR_NAME) {
                            abort!(attr.span(), "Default functions are not yet supported");
                            // if specific.is_none() {
                            //     specific = Some((idx, input_clone.clone(), FillTypeAttr::Function(attr.clone())));
                            // } else {
                            //     abort!(attr.span(), "Duplicate attribute for default argument");
                            // }
                        } else {
                            abort!(ty.span(), "Proper handling of other attributes are not yet supported");
                        }
                    }
                    if let Some(specific) = specific {
                        first_default_found = true;
                        let idx = specific.0;
                        defaultable_arguments.push(specific);
                        // clear the attribute too
                        ty.attrs.remove(idx);
                    } else {
                        non_default_arguments.push(input.clone());
                    }
                }
            },
        }
    }

    let mut call_args = vec![];
    let mut expr_args = vec![];

    let num_non_default_args = non_default_arguments.len();
    
    for (i, _arg) in non_default_arguments.into_iter().enumerate() {
        let expr = make_named_ident_and_expr_for_macro(i);
        let call_arg = make_named_ident_for_macro(i);
        
        expr_args.push(expr);
        call_args.push(call_arg);
    }

    let mut call_args_list = proc_macro2::TokenStream::new();
    call_args_list.append_separated(
        &call_args,
        proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
    );

    let mut expr_args_list = proc_macro2::TokenStream::new();
    expr_args_list.append_separated(
        expr_args,
        proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
    );

    let original_ident = parsed_fn.sig.ident.clone();
    parsed_fn.sig.ident = Ident::new(&format!("{}_impl", parsed_fn.sig.ident), Span::call_site());
    let modified_ident = parsed_fn.sig.ident.clone();

    let mut named_macro_exprs = vec![];

    for (i, full_arg_expr, _attr) in defaultable_arguments.iter() {
        // create macro params like c = $w: expr for full function calls

        let mut named_macro_argument = proc_macro2::TokenStream::new();
        let arg_name = match full_arg_expr {
            FnArg::Typed(ref ty) => {
                ty.pat.clone()
            },
            _ => unreachable!()
        };

        let parts = vec![quote!(#arg_name), make_named_ident_and_expr_for_macro(*i + num_non_default_args)];
        named_macro_argument.append_separated(
            parts,
            proc_macro2::Punct::new('=', proc_macro2::Spacing::Alone),
        );
        named_macro_exprs.push(named_macro_argument);
    }

    // create all possible options of named argumentst for defaultable parameters
    let mut cases = vec![];
    extend_call_variants(
        &defaultable_arguments, 
        &named_macro_exprs,
        &mut cases, 
        vec![], 
        vec![], 
        0,
        num_non_default_args,
    );
    
    let mut full_call_args_lists = vec![];
    let mut full_macro_args_lists = vec![];
    for (call_args, macro_args) in cases.into_iter() {
        let mut macro_args_parts = vec![expr_args_list.clone()];
        macro_args_parts.extend(macro_args);
        let mut full_macro_args = proc_macro2::TokenStream::new();
        full_macro_args.append_separated(
            macro_args_parts,
            proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
        );

        let mut call_args_parts = vec![call_args_list.clone()];
        call_args_parts.extend(call_args);
        let mut full_call_args = proc_macro2::TokenStream::new();
        full_call_args.append_separated(
            call_args_parts,
            proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
        );

        // eprintln!("Call args = {}", full_call_args.to_string());
        // eprintln!("Macro args = {}", full_macro_args.to_string());

        full_call_args_lists.push(full_call_args);
        full_macro_args_lists.push(full_macro_args);
    }
    // now crate a set of macro that will have a proper structure

    let mut inner_macro_quote: Option<TokenStream2> = None;
    for (macro_args, call_args) in full_macro_args_lists.into_iter().zip(full_call_args_lists.into_iter()) {
        if let Some(inner_macro_quote_content) = inner_macro_quote.take() {
            let q = quote!(
                #inner_macro_quote_content;
    
                (#macro_args) => {
                    #modified_ident(#call_args)
                }
            );
            inner_macro_quote = Some(q);
        } else {
            inner_macro_quote = Some(quote!(
                (#macro_args) => {
                    #modified_ident(#call_args)
                }
            ));
        }

    }

    // we either fill a value with a default expression, or with a function

    let macro_quote = quote! {
        #[macro_export]
        macro_rules! #original_ident {
            #inner_macro_quote
        }
    };

    // eprintln!("Macro = {}", macro_quote.to_string());

    // eprintln!("Called from {:?} inside module path {:?}", file!(), module_path!());

    quote!(
        // macro

        #macro_quote

        // impl

        #parsed_fn
    ).into()
}

fn make_named_ident_for_macro(idx: usize) -> proc_macro2::TokenStream {
    let tmp = "macro_rules! test{($w: expr) => {}}";
    let example_expr = syn::parse_str::<syn::ItemMacro>(tmp).unwrap();
    let base_group = example_expr.mac.tokens.into_iter().next().unwrap();
    let base_group = match base_group {
        proc_macro2::TokenTree::Group(g) => g,
        _ => unreachable!()
    };
    let stream = base_group.stream();
    let mut items = vec![];
    for item in stream.into_iter() {
        let mut item = item;
        item.set_span(Span::call_site());
        items.push(item);
    }

    let mut call_items = items[..2].to_vec();
    match &mut call_items[1] {
        proc_macro2::TokenTree::Ident(ref mut idt) => {
            *idt = Ident::new(&format!("w{}", idx), Span::call_site());
        },
        _ => {
            unreachable!()
        }
    };

    let mut call_arg = proc_macro2::TokenStream::new();
    call_arg.extend(call_items);

    call_arg
}


fn make_named_ident_and_expr_for_macro(idx: usize) -> proc_macro2::TokenStream {
    let tmp = "macro_rules! test{($w: expr) => {}}";
    let example_expr = syn::parse_str::<syn::ItemMacro>(tmp).unwrap();
    let base_group = example_expr.mac.tokens.into_iter().next().unwrap();
    let base_group = match base_group {
        proc_macro2::TokenTree::Group(g) => g,
        _ => unreachable!()
    };
    let stream = base_group.stream();
    let mut items = vec![];
    for item in stream.into_iter() {
        let mut item = item;
        item.set_span(Span::call_site());
        items.push(item);
    }

    match &mut items[1] {
        proc_macro2::TokenTree::Ident(ref mut idt) => {
            *idt = Ident::new(&format!("w{}", idx), Span::call_site());
        },
        _ => {
            unreachable!()
        }
    };

    let mut expr_arg = proc_macro2::TokenStream::new();
    expr_arg.extend(items);

    expr_arg
}

fn extend_call_variants(
    defaultable_arguments: &Vec<(usize, FnArg, FillTypeAttr)>, 
    named_macro_args: & Vec<TokenStream2>,
    cases: &mut Vec<(Vec<TokenStream2>, Vec<TokenStream2>)>,
    mut call_args_chain: Vec<TokenStream2>,
    mut macro_args_chain: Vec<TokenStream2>,
    skip: usize,
    num_non_default_params: usize,
) {
    if skip == 0 {
        call_args_chain = vec![];
        macro_args_chain = vec![];
    }
    for (_i, _full_arg_expr, attr) in defaultable_arguments.iter().skip(skip) {
        // missing
        {
            let mut call_args = call_args_chain.clone();
            let macro_args = macro_args_chain.clone();
            // nothing into the chain in macro expansion,
            // and place default expression into the function call place

            match attr {
                FillTypeAttr::Expression(ref expr) => {
                    let args = expr.parse_args();
                    if args.is_err() {
                        abort!(expr.span(), "Failed to parse an expression");
                    }
                    let args: TokenStream2 = args.unwrap();
                    call_args.push(args);
                },
                FillTypeAttr::Function(ref func) => {
                    abort!(func.span(), "Functions are not supported yet");
                }
            }

            let next_skip = skip + 1;
            if next_skip == defaultable_arguments.len() {
                cases.push((call_args, macro_args));
            } else {
                extend_call_variants(
                    defaultable_arguments,
                    named_macro_args,
                    cases,
                    call_args,
                    macro_args,
                    next_skip,
                    num_non_default_params
                );
            }
        }
        // named like c = $w10: expr in macro
        // and $w10 in function call
        {
            let mut call_args = call_args_chain.clone();
            let mut macro_args = macro_args_chain.clone();
            // nothing into the chain in macro expansion,
            // and place default expression into the function call place

            macro_args.push(named_macro_args[skip].clone());
            call_args.push(make_named_ident_for_macro(skip + num_non_default_params));

            let next_skip = skip + 1;
            if next_skip == defaultable_arguments.len() {
                cases.push((call_args, macro_args));
            } else {
                extend_call_variants(
                    defaultable_arguments,
                    named_macro_args,
                    cases,
                    call_args,
                    macro_args,
                    next_skip,
                    num_non_default_params
                );
            }
        }
    }
}