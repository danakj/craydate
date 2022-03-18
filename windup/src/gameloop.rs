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

pub struct GameObj {
  bitmap: LCDBitmap,
  pos: euclid::default::Rect<i32>,
  // stuff that doesn't fit into integer pos
  pos_remainder: euclid::default::Vector2D<f32>,
  vel: euclid::default::Vector2D<f32>,
  // FIXME: this shouldn't live here, refactor into a better input handler
  crank_accum: f32,
}
impl GameObj {
  // FIXME: this is really the player's update
  pub fn update(&mut self, inputs: &Inputs) {
    if let &Crank::Undocked { angle: _, change } = inputs.crank() {
      self.crank_accum += change;
      self.crank_accum = fmax(fmin(self.crank_accum, CRANK_MAX), -CRANK_MAX);
    }

    for (button, event) in inputs.buttons().all_events() {
      if event != ButtonEvent::Push {
        continue;
      }
      if button == Button::Left {
        self.pos = self.pos.translate(euclid::default::Vector2D::new(-1, 0));
      }
      if button == Button::Right {
        self.pos = self.pos.translate(euclid::default::Vector2D::new(1, 0));
      }
      if button == Button::A {
        self.pos.origin.x = INITIAL_X;
      }
      // Only can jump when on the ground.
      if button == Button::Up && self.pos.min_y() == FLOOR_Y {
        let crank_sign: f32 = if self.crank_accum < 0.0 { -1.0 } else { 1.0 };

        let crank: f32 = fmin(self.crank_accum.abs() / CRANK_FACTOR, CRANK_MAX).into();

        self.crank_accum = 0.0;
        self.vel = euclid::default::Vector2D::new(crank * crank_sign, -crank)
      }
    }

    self.vel.y += GRAVITY;

    let mut remain = self.pos_remainder;
    remain += self.vel;
    let rounded = remain.round();
    self.pos = self.pos.translate(rounded.to_i32());
    self.pos_remainder = remain - rounded;

    // """collision"""
    if self.pos.origin.y >= FLOOR_Y {
      self.pos.origin.y = FLOOR_Y;
      self.vel = euclid::default::Vector2D::new(0.0, 0.0);
    }
    if self.pos.origin.x < MIN_X {
      self.pos.origin.x = MIN_X;
    }
    if self.pos.origin.x > MAX_X {
      self.pos.origin.x = MAX_X;
    }
  }

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

  let player = GameObj {
    bitmap: graphics.load_bitmap("images/bot").unwrap(),
    pos: euclid::rect(INITIAL_X, FLOOR_Y, 32, 32),
    pos_remainder: euclid::vec2(0.0, 0.0),
    vel: euclid::vec2(0.0, 0.0),
    crank_accum: 0.0,
  };

  let mut objs = [player];

  let fw = system.frame_watcher();
  loop {
    let inputs = fw.next().await;

    // TODO: probably need a more efficient drawing mechanism than full redraw
    graphics.clear(LCDSolidColor::kColorWhite);

    for obj in &mut objs {
      obj.update(&inputs);
    }

    for obj in &objs {
      obj.draw(graphics);
    }

    let hacky_intrusive_crank_value = objs[0].crank_accum;

    graphics.draw_text(
      "turn crank, hit up to jump",
      PDStringEncoding::kASCIIEncoding,
      5,
      0,
    );
    graphics.draw_text("hit A to reset", PDStringEncoding::kASCIIEncoding, 5, 15);

    let crank_str = format!("{:.1}", hacky_intrusive_crank_value);
    graphics.draw_text(crank_str, PDStringEncoding::kASCIIEncoding, 5, 30);
    graphics.draw_fps(400 - 15, 0);
  }
}
