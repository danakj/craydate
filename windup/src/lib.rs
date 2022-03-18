#![no_std]
#![deny(clippy::all)]
#![feature(never_type)]

mod playground;

#[playdate::main]
async fn main(mut api: playdate::Api) -> ! {
  playground::run(api).await
}
