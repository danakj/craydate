extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let func = parse_macro_input!(item as ItemFn);
  let func_ident = &func.sig.ident;

  if func.sig.asyncness.is_none() {
    return quote_spanned! { func.sig.span()  =>
      compile_error!{"The #[playdate::main] function must be async."}
    }
    .into();
  }

  quote!{
    mod __main {
      use super::*;
      use ::core::pin::Pin;
      use ::core::future::Future;
      use ::playdate::macro_helpers::*;

      #[no_mangle]
      extern "C" fn eventHandler(eh1: EventHandler1, eh2: EventHandler2, eh3: EventHandler3) -> i32 {
        fn main_wrapper(api: SafeApi) -> Pin<Box<dyn Future<Output = !>>> {
          Box::pin(#func_ident(api))
        }
        let config = GameConfig {
          main_fn: main_wrapper,
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
