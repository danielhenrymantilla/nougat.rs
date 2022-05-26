//! Crate not intended for direct use.
//! Use https:://docs.rs/nougat instead.
// Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template
#![allow(nonstandard_style, unused_imports)]

use ::core::{
    mem,
    ops::Not as _,
};
use ::proc_macro::{
    TokenStream,
};
use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
    TokenTree as TT,
};
use ::quote::{
    format_ident,
    quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::{Pair, Punctuated},
    Result, // Explicitly shadow it
    spanned::Spanned,
};

#[path = "adju-gat-e.rs"]
mod adjugate;

#[path = "Gat-bang.rs"]
mod Gat;

use self::utils::*;
mod utils;

// // Documentation located in the frontend crate.
// #[proc_macro_attribute] pub
// fn adjugate (
//     attrs: TokenStream,
//     input: TokenStream,
// ) -> TokenStream
// {
//     adjugate::adjugate(
//         parse_macro_input!(attrs),
//         parse_macro_input!(input),
//     )
//     .into_token_stream()
//     .into()
// }

// Documentation located in the frontend crate.
#[proc_macro_attribute] pub
fn gat (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    unwrap("#[::nougat::gat]", {
        #[path = "gat-attr/_mod.rs"]
        mod implementation;
        implementation::gat(attrs.into(), input.into())
    })
}

// Documentation located in the frontend crate.
#[proc_macro] pub
fn Gat (
    input: TokenStream,
) -> TokenStream
{
    unwrap("::nougat::Gat!", {
        parse(input).and_then(Gat::Gat::<Error>)
    })
}

fn combine_trait_name_and_assoc_type (
    trait_name: &'_ Ident,
    assoc_type: &'_ Ident,
) -> Ident
{
    Ident::new(
        &format!("{}__{}", trait_name, assoc_type),
        assoc_type.span(), // .located_at(trait_name.span()),
    )
}
