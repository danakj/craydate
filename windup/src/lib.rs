#![no_std]
#![deny(clippy::all)]
#![feature(never_type)]

mod gameloop;
mod playground;

#[playdate::main]
async fn main(api: playdate::Api) -> ! {
  // TODO: could we use a different build target for this??
  playground::_run(api).await;

  gameloop::run(api).await
}
