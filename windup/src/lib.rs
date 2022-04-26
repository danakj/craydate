#![no_std]
#![deny(clippy::all)]
#![feature(never_type)]

extern crate alloc;

mod gameloop;
mod playground;

#[craydate::main]
async fn main(api: craydate::Api) -> ! {
  // TODO: could we use a different build target for this??
  // playground::_run(api).await;

  gameloop::run(api).await
}
