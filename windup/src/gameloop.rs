use alloc::vec::Vec;
use core::cmp;

use float_ord::FloatOrd;
use num_traits::float::FloatCore;
use playdate::*;

const INITIAL_X: i32 = 150;
const MIN_X: i32 = 0;
const MAX_X: i32 = 400 - 32;
const FLOOR_Y: i32 = 200;
// delta velocity per second
const GRAVITY: f32 = 3.0;

// larger = more crank required
const CRANK_FACTOR: f32 = 15.0;
// maximum crank, before factor applied
const CRANK_MAX: f32 = 450.0;

pub fn fmax(v1: f32, v2: f32) -> f32 {
  // FIXME: surely there is an easier way to write this
  cmp::max::<FloatOrd<f32>>(FloatOrd::<f32>(v1), FloatOrd::<f32>(v2)).0
}

pub fn fmin(v1: f32, v2: f32) -> f32 {
  // FIXME: surely there is an easier way to write this
  cmp::min::<FloatOrd<f32>>(FloatOrd::<f32>(v1), FloatOrd::<f32>(v2)).0
}

pub struct AccumInputs {
  crank_accum: f32,
}
impl AccumInputs {
  pub fn accumulate(&mut self, inputs: &Inputs) {
    if let &Crank::Undocked { angle: _, change } = inputs.crank() {
      self.crank_accum += change;
      self.crank_accum = fmax(fmin(self.crank_accum, CRANK_MAX), -CRANK_MAX);
    }
  }
}

pub struct World {
  player: GameObj,
  blocks: Vec<[i32; 2]>,
  block_bmp: LCDBitmap,
  // TODO: add other stuff in the world
}
impl World {
  fn player_update(&mut self, inputs: &Inputs, accum: &mut AccumInputs) {
    let player = &mut self.player;
    if inputs.buttons().left_state() == ButtonState::Pushed {
      player.pos = player.pos.translate(euclid::default::Vector2D::new(-1, 0));
    }
    if inputs.buttons().right_state() == ButtonState::Pushed {
      player.pos = player.pos.translate(euclid::default::Vector2D::new(1, 0));
    }
    for (button, event) in inputs.buttons().all_events() {
      if event != ButtonEvent::Push {
        continue;
      }
      if button == Button::A {
        player.pos.origin.x = INITIAL_X;
      }
      // Only can jump when on the ground.
      if button == Button::Up && player.pos.min_y() == FLOOR_Y {
        let crank_sign: f32 = if accum.crank_accum < 0.0 { -1.0 } else { 1.0 };

        let crank: f32 = fmin(accum.crank_accum.abs() / CRANK_FACTOR, CRANK_MAX).into();

        accum.crank_accum = 0.0;
        player.vel = euclid::default::Vector2D::new(crank * crank_sign, -crank)
      }
    }

    player.vel.y += GRAVITY;

    let mut remain = player.pos_remainder;
    remain += player.vel;
    let rounded = remain.round();
    player.pos = player.pos.translate(rounded.to_i32());
    player.pos_remainder = remain - rounded;

    // """collision"""
    if player.pos.origin.y >= FLOOR_Y {
      player.pos.origin.y = FLOOR_Y;
      player.vel = euclid::default::Vector2D::new(0.0, 0.0);
    }
    if player.pos.origin.x < MIN_X {
      player.pos.origin.x = MIN_X;
    }
    if player.pos.origin.x > MAX_X {
      player.pos.origin.x = MAX_X;
    }
  }

  pub fn update(&mut self, inputs: &Inputs, accum: &mut AccumInputs) {
    self.player_update(inputs, accum);
  }

  pub fn draw(&self, g: &mut Graphics) {
    for block in &self.blocks {
      g.draw_bitmap(&self.block_bmp, block[0] * 32, block[1] * 32, LCDBitmapFlip::kBitmapUnflipped);
    }
    // TODO: draw other stuff in world
    self.player.draw(g);
  }
}

pub struct GameObj {
  bitmap: LCDBitmap,
  pos: euclid::default::Rect<i32>,
  // stuff that doesn't fit into integer pos
  pos_remainder: euclid::default::Vector2D<f32>,
  vel: euclid::default::Vector2D<f32>,
}
impl GameObj {
  fn draw(&self, g: &mut Graphics) {
    g.draw_bitmap(
      &self.bitmap,
      self.pos.min_x(),
      self.pos.min_y(),
      LCDBitmapFlip::kBitmapUnflipped,
    );
  }
}

pub async fn run(mut api: playdate::Api) -> ! {
  let system = &mut api.system;
  let graphics = &mut api.graphics;

  let mut world = World {
    player: GameObj {
      bitmap: graphics.load_bitmap("images/bot").unwrap(),
      pos: euclid::rect(INITIAL_X, FLOOR_Y, 32, 32),
      pos_remainder: euclid::vec2(0.0, 0.0),
      vel: euclid::vec2(0.0, 0.0),
    },
    blocks: Vec::from([
      [0, 6],
      [1, 6],
      [2, 6],
      [3, 6],
    ]),
    block_bmp: graphics.load_bitmap("images/box").unwrap(),
  };

  let mut accum = AccumInputs { crank_accum: 0.0 };

  let fw = system.frame_watcher();
  loop {
    let inputs = fw.next().await;

    // TODO: probably need a more efficient drawing mechanism than full redraw
    graphics.clear(LCDSolidColor::kColorWhite);

    accum.accumulate(&inputs);
    world.update(&inputs, &mut accum);
    world.draw(graphics);

    graphics.draw_text(
      "turn crank, hit up to jump",
      PDStringEncoding::kASCIIEncoding,
      5,
      0,
    );
    graphics.draw_text("hit A to reset", PDStringEncoding::kASCIIEncoding, 5, 15);

    let crank_str = format!("{:.1}", accum.crank_accum);
    graphics.draw_text(&crank_str, PDStringEncoding::kASCIIEncoding, 5, 30);
    graphics.draw_fps(400 - 15, 0);
  }
}
