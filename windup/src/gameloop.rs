use alloc::vec::Vec;
use core::cmp;

use float_ord::FloatOrd;
use num_traits::float::FloatCore;
use playdate::*;

const INITIAL_X: i32 = 50;
const INITIAL_Y: i32 = 50;
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
  blocks: Vec<euclid::default::Rect<i32>>,
  block_bmp: Bitmap,
  // TODO: add other stuff in the world
}
impl World {
  fn player_update(&mut self, inputs: &Inputs, accum: &mut AccumInputs, _system: &mut System) {
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
        player.pos.origin.y = INITIAL_Y;
      }
      // Only can jump when on the ground.
      if button == Button::Up && player.grounded {
        let crank_sign: f32 = if accum.crank_accum < 0.0 { -1.0 } else { 1.0 };

        let crank: f32 = fmin(accum.crank_accum.abs() / CRANK_FACTOR, CRANK_MAX).into();

        accum.crank_accum = 0.0;
        player.vel = euclid::default::Vector2D::new(crank * crank_sign, -crank);

        player.grounded = false;
      }
    }

    // Acceleration.
    player.vel.y += GRAVITY;

    // Apply velocity to find potential new positions.
    let mut remain = player.pos_remainder;
    remain += player.vel;
    let rounded = remain.round();
    let mut new_pos = player.pos.translate(rounded.to_i32());
    let mut new_pos_remainder = remain - rounded;
    let mut new_grounded = player.grounded;

    // Object collision to adjust velocity.
    for block in &self.blocks {
      if !new_pos.intersects(block) {
        continue;
      }
      // Extremely basic penetration detection / reversal along the shortest axis.
      let x_depth: i32;
      let y_depth: i32;
      if new_pos.max_x() > block.min_x() && new_pos.max_x() < block.max_x() {
        x_depth = new_pos.max_x() - block.min_x();
      } else if new_pos.min_x() < block.max_x() && new_pos.min_x() > block.min_x() {
        x_depth = new_pos.min_x() - block.max_x();
      } else {
        x_depth = 0;
      }
      if new_pos.max_y() > block.min_y() && new_pos.max_y() < block.max_y() {
        y_depth = new_pos.max_y() - block.min_y();
      } else if new_pos.min_y() < block.max_y() && new_pos.min_y() > block.min_y() {
        y_depth = new_pos.min_y() - block.max_y();
      } else {
        y_depth = 0;
      }
      // _system.log(format!("p: {:?}, b: {:?}, x: {:?}, y: {:?}", player.pos, block, x_depth, y_depth));
      if num_traits::abs(x_depth) < num_traits::abs(y_depth) {
        player.vel.x = 0.0;
        new_pos.origin.x -= x_depth;
        new_pos_remainder.x = 0.0;
      } else if y_depth != 0 {
        player.vel.y = 0.0;
        new_pos.origin.y -= y_depth;
        new_pos_remainder.y = 0.0;
        if y_depth > 0 {
          new_grounded = true;
          player.vel.x = 0.0;
        }
      }
    }

    player.pos = new_pos;
    player.pos_remainder = new_pos_remainder;
    player.grounded = new_grounded;

    // Hard collision.
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

  pub fn update(&mut self, inputs: &Inputs, accum: &mut AccumInputs, system: &mut System) {
    self.player_update(inputs, accum, system);
  }

  pub fn draw(&self, g: &mut Graphics) {
    for block in &self.blocks {
      g.draw_bitmap(
        &self.block_bmp,
        block.origin.x,
        block.origin.y,
        BitmapFlip::kBitmapUnflipped,
      );
    }
    // TODO: draw other stuff in world
    self.player.draw(g);
  }
}

pub struct GameObj {
  bitmap: Bitmap,
  pos: euclid::default::Rect<i32>,
  // stuff that doesn't fit into integer pos
  pos_remainder: euclid::default::Vector2D<f32>,
  vel: euclid::default::Vector2D<f32>,
  grounded: bool,
}
impl GameObj {
  fn draw(&self, g: &mut Graphics) {
    g.draw_bitmap(
      &self.bitmap,
      self.pos.min_x(),
      self.pos.min_y(),
      BitmapFlip::kBitmapUnflipped,
    );
  }
}

pub async fn run(mut api: playdate::Api) -> ! {
  let system = &mut api.system;
  let graphics = &mut api.graphics;

  let mut world = World {
    player: GameObj {
      bitmap: graphics.load_bitmap("images/bot").unwrap(),
      pos: euclid::rect(INITIAL_X, INITIAL_Y, 20, 20),
      pos_remainder: euclid::vec2(0.0, 0.0),
      vel: euclid::vec2(0.0, 0.0),
      grounded: false,
    },
    blocks: Vec::from(
      [
        // the floor
        [0, 6],
        [1, 6],
        [2, 6],
        [3, 6],
        [4, 6],
        [5, 6],
        [6, 6],
        [7, 6],
        [8, 6],
        [9, 6],
        [10, 6],
        [11, 6],
        [12, 6],
        // small bump
        [4, 5],
        // large bump
        [7, 5],
        [7, 4],
        [7, 3],
      ]
      .map(|[x, y]| euclid::rect(x * 32, y * 32, 32, 32))
    ),
    block_bmp: graphics.load_bitmap("images/box").unwrap(),
  };

  let mut accum = AccumInputs { crank_accum: 0.0 };

  let fw = system.frame_watcher();
  loop {
    let inputs = fw.next().await;

    // TODO: probably need a more efficient drawing mechanism than full redraw
    graphics.clear(SolidColor::kColorWhite);

    accum.accumulate(&inputs);
    world.update(&inputs, &mut accum, system);
    world.draw(graphics);

    graphics.draw_text(
      "turn crank, hit up to jump",
      StringEncoding::kASCIIEncoding,
      5,
      0,
    );
    graphics.draw_text("hit A to reset", StringEncoding::kASCIIEncoding, 5, 15);

    let crank_str = format!("{:.1}", accum.crank_accum);
    graphics.draw_text(&crank_str, StringEncoding::kASCIIEncoding, 5, 30);
    graphics.draw_fps(400 - 15, 0);
  }
}
