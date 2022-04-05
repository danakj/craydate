use playdate::*;

/// A testing function to dump new functionality into for manual verification.
pub async fn _run(mut api: playdate::Api) -> ! {
  let system = &api.system;
  let graphics = &mut api.graphics;

  let mut grey50 = Bitmap::new(8, 8, SolidColor::kColorBlack);
  for x in (0..8).step_by(2) {
    for y in 0..8 {
      let xwrite = x + y % 2;
      let ywrite = y;
      grey50.pixels_mut().set(xwrite, ywrite, true)
    }
  }
  let grey50 = Pattern::from_bitmap(&grey50, 0, 0);
  graphics.clear(&grey50);

  let mut bmp = Bitmap::new(100, 40, SolidColor::kColorWhite);
  let mask = Bitmap::new(100, 40, SolidColor::kColorWhite);
  bmp.set_mask_bitmap(&mask).expect("mask problems");

  graphics.draw_bitmap(&bmp, 5, 9, BitmapFlip::kBitmapUnflipped);

  let mut stencil = Bitmap::new(64, 64, SolidColor::kColorWhite);
  for y in 0..64 as usize {
    let c = y % 4 != 0;
    for x in 0..64 as usize {
      stencil.pixels_mut().set(x, y, c);
    }
  }

  let font = Font::from_file("fonts/Mini Sans 2X/Mini Sans 2X.pft");
  let _active = match &font {
    Ok(font) => {
      api.system.log(format!("Font height: {}", font.font_height()));

      let page = font.font_page('d');
      api.system.log("Got page");
      let _bitmap = page.glyph('d').unwrap().bitmap();

      Some(graphics.set_font(font))
    }
    Err(e) => {
      api.system.log(format!("ERROR: loading font {}", e));
      None
    }
  };

  {
    let _stencil_holder = graphics.set_stencil(&stencil);
    graphics.draw_text("Bloop", StringEncoding::kASCIIEncoding, 30, 20);
  }

  let mut copy = graphics.working_frame_bitmap();

  for y in 20..30 {
    for x in 10..20 {
      copy.pixels_mut().set(x, y, false);
    }
  }
  graphics.draw_bitmap(&copy, 0, 30, BitmapFlip::kBitmapUnflipped);

  let points = [
    euclid::default::Point2D::new(10, 10),
    euclid::default::Point2D::new(20, 20),
    euclid::default::Point2D::new(10, 30),
    euclid::default::Point2D::new(0, 20),
  ];
  graphics.fill_polygon(
    &points,
    Color::Solid(SolidColor::kColorBlack),
    PolygonFillRule::kPolygonFillEvenOdd,
  );

  let c = graphics.bitmaps_collide(
    BitmapCollider {
      bitmap: &bmp,
      flipped: BitmapFlip::kBitmapUnflipped,
      x: 0,
      y: 0,
    },
    BitmapCollider {
      bitmap: &copy,
      flipped: BitmapFlip::kBitmapUnflipped,
      x: 0,
      y: 0,
    },
    euclid::rect(0, 0, 100, 100),
  );
  system.log(format!("collision: {}", c));

  // working image
  let yo_path = "images/yo";
  let load = Bitmap::from_file(yo_path);
  if let Ok(bitmap) = load {
    graphics.draw_bitmap(&bitmap, 100, 80, BitmapFlip::kBitmapUnflipped);
  }

  // broken image
  let broken_path = "images/wat";
  let load = Bitmap::from_file(broken_path);
  if let Err(error) = load {
    system.log(error);
  }

  let display = &mut api.display;
  display.set_inverted(true);
  //display.set_flipped(true, false);
  display.set_scale(2);

  let list_files_in = |path: &str| match api.file.list_files(path) {
    Ok(files) => {
      api.system.log(format!("{}/ files:", path));
      for fname in files {
        api.system.log(format!("  {:?}", fname))
      }
    }
    Err(e) => api.system.log(format!("ERROR: {}", e)),
  };
  let make_dir = |path: &str| match api.file.make_folder(path) {
    Ok(()) => system.log(format!("mkdir {}", path)),
    Err(e) => system.log(e),
  };
  let rename = |from: &str, to: &str| match api.file.rename(from, to) {
    Ok(()) => {
      system.log(format!("renamed {} to {}", from, to));
      list_files_in("myfolder");
    }
    Err(e) => system.log(e),
  };
  let delete_recursive = |path: &str| match api.file.delete_recursive(path) {
    Ok(()) => system.log(format!("deleted {} recursive", path)),
    Err(e) => system.log(e),
  };
  let stat = |path: &str| match api.file.stat(path) {
    Ok(stats) => system.log(format!("stat {}: {:?}", path, stats)),
    Err(e) => system.log(e),
  };
  let write_file = |path: &str, stuff: &[u8]| match api.file.write_file(path, stuff) {
    Ok(()) => system.log(format!("wrote {}", path)),
    Err(e) => system.log(e),
  };
  let read_file = |path: &str| match api.file.read_file(path) {
    Ok(content) => system.log(format!("read {}: {:?}", path, String::from_utf8(content))),
    Err(e) => system.log(e),
  };

  list_files_in("images");

  make_dir("myfolder");
  make_dir("myfolder/two");
  list_files_in("myfolder");
  list_files_in("myfolder/two");

  rename("myfolder/two", "myfolder/three");
  stat("myfolder/three");

  write_file("myfolder/three", b"bees\n");
  write_file("myfolder/three/bears.txt", b"want honey\n");
  read_file("myfolder/three/bears.txt");
  read_file("myfolder/three/no_bears.txt");

  delete_recursive("myfolder");

  let vol = api.sound.default_channel().volume();
  api.system.log(format!("Default channel volume (in 0-1): {}", vol));

  let mut i32callbacks = Callbacks::<(i32, &System)>::new();

  let mut fileplayer = FilePlayer::from_file("sounds/mojojojo.pda");
  api.sound.default_channel_mut().attach_source(&mut fileplayer);
  fileplayer.set_buffer_len(TimeTicks::from_milliseconds(200));
  api.system.log(format!(
    "Fileplayer length: {} seconds",
    fileplayer.file_len().to_seconds(),
  ));
  fileplayer.as_mut().set_completion_callback(&mut i32callbacks, |(i, system)| {
    system.log(format!("finished playback of mojojojo {}", i));
  });
  fileplayer.play(1);

  let action_item = MenuItem::new_action("hello world", &mut i32callbacks, |(i, system)| {
    system.log(format!("menu action {}", i));
  });
  action_item.title();
  let check_item = MenuItem::new_checkmark("dank", false, &mut i32callbacks, |(i, system)| {
    system.log(format!("dankness adjusted {}", i));
  });
  check_item.set_checked(true);
  let options_item = MenuItem::new_options(
    "temp",
    ["too hot", "too cold", "just right"],
    &mut i32callbacks,
    |(i, system)| {
      system.log(format!("temperature adjusted {}", i));
    },
  );
  options_item.set_value(2);

  system.log(format!(
    "Entering main loop at time {}",
    api.system.current_time()
  ));
  let events = system.system_event_watcher();
  loop {
    let (inputs, frame_number) = match events.next().await {
      SystemEvent::NextFrame {
        inputs,
        frame_number,
      } => (inputs, frame_number),
      SystemEvent::WillLock => {
        api.system.log("locked");
        continue;
      }
      SystemEvent::DidUnlock => {
        api.system.log("unlocked");
        continue;
      }
      SystemEvent::Callback => {
        i32callbacks.run((1, &api.system));
        continue;
      }
      _ => continue,
    };
    for (button, event) in inputs.buttons().all_events() {
      match event {
        playdate::ButtonEvent::Push => {
          api.system.log(format!("{:?} pushed on frame {}", button, frame_number));
        }
        playdate::ButtonEvent::Release => {
          api.system.log(format!("{:?} released on frame {}", button, frame_number));
        }
      }
    }

    api.graphics.draw_fps(400 - 15, 0);
  }
}
