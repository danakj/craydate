extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let func = item;// as syn::ItemFn;

  quote!{
    mod __main {
      use super::*;
      use ::playdate::macro_helpers::*;

      #[no_mangle]
      pub extern "C" fn eventHandler(eh1: EventHandler1, eh2: EventHandler2, eh3: EventHandler3) -> i32 {

            
        playdate::macro_helpers::initialize(eh1, eh2, eh3);
        0  // What does it do? We don't know.
      }

      #[cfg(all(target_arch = "arm", target_os = "none"))]
      type EventHandlerFn = extern "C" fn(EventHandler1, EventHandler2, EventHandler3) -> i32;

      #[cfg(all(target_arch = "arm", target_os = "none"))]
      #[used]
      #[link_section = ".capi_handler"]
      static EVENT_HANDLER: EventHandlerFn = eventHandler;

      //#func
    }
  }.into()
}