extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let func = parse_macro_input!(item as ItemFn);
  let func_ident = func.sig.ident.clone();

  quote!{
    mod __main {
      use super::*;
      use ::playdate::macro_helpers::*;

      #[no_mangle]
      extern "C" fn eventHandler(eh1: EventHandler1, eh2: EventHandler2, eh3: EventHandler3) -> i32 {

        let config = GameConfig {
          main_fn: #func_ident,
        };
        initialize(eh1, eh2, eh3, config);
        0  // What does it do? We don't know.
      }

      #[cfg(all(target_arch = "arm", target_os = "none"))]
      type EventHandlerFn = extern "C" fn(EventHandler1, EventHandler2, EventHandler3) -> i32;

      #[cfg(all(target_arch = "arm", target_os = "none"))]
      #[used]
      #[link_section = ".capi_handler"]
      static EVENT_HANDLER: EventHandlerFn = eventHandler;

      #func
    }
  }.into()
}