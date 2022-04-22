//! This crate provides a `#[dynpath()]` macro that can be placed a `mod` statement and which points
//! the module to a dynamic path.
//!
//! The macro takes a single parameter which is the name of an environment variable to read the path
//! from, and it appends the module name and `.rs` extension onto the contents of the variable.
//!
//! # Example
//! ```
//! // Turns into `#[path = "whatever/is/in/OUT_DIR/bindings.rs"]`.
//! #[dynpath("OUT_DIR")]
//! mod bindings;
//! ```

#![deny(clippy::all)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::*;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

macro_rules! tokens {
    ($($expr:expr),* $(,)?) => {
        vec![$($expr,)*].into_iter().collect::<TokenStream>()
    }
}

/// See the crate documentation for how to use the `#[dynpath()]` macro.
#[proc_macro_attribute]
pub fn dynpath(attr: TokenStream, item: TokenStream) -> TokenStream {
  let attr = parse_macro_input!(attr as syn::AttributeArgs);
  if attr.len() != 1 {
    return quote! {
      compile_error!("Expected one argument.")
    }
    .into();
  }

  let option = match &attr[0] {
    syn::NestedMeta::Lit(syn::Lit::Str(lit)) => lit.value(),
    _ => {
      return quote! {
          compile_error!("Argument should be the name of an environment variable, e.g. `\"OUT_DIR\"`")
      }
      .into();
    }
  };

  let dir = std::env::var(&option)
    .unwrap_or_else(|_| panic!("The \"{}\" environment variable is not set", option));

  let item = parse_macro_input!(item as syn::ItemMod);
  let modname = item.ident.to_string();

  let modpath = std::path::PathBuf::from(dir).join(format!("{}.rs", modname));

  let stream = vec![
    TokenTree::Punct(Punct::new('#', Spacing::Alone)),
    TokenTree::Group(Group::new(
      Delimiter::Bracket,
      tokens![
        TokenTree::Ident(Ident::new("path", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::string(&modpath.to_string_lossy())),
      ],
    )),
  ];

  let item_stream: TokenStream = item.to_token_stream().into();

  stream.into_iter().chain(item_stream.into_iter()).collect()
}
