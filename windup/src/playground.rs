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
      grey50.as_pixels_mut().set(xwrite, ywrite, PixelColor::WHITE)
    }
  }
  let _grey50 = Pattern::from_bitmap(&grey50, 0, 0);
  let mut grey50_colors = [PixelColor::BLACK; 8 * 8];
  for x in 0..8 {
    for y in 0..8 {
      let xodd = x % 2 != 0;
      let yodd = y % 2 != 0;
      if yodd == xodd {
        grey50_colors[y * 8 + x] = PixelColor::WHITE;
      }
    }
  }
  let grey50 = Pattern::new_unmasked(grey50_colors);
  graphics.clear(&grey50);

  let mut bmp = Bitmap::new(100, 40, SolidColor::kColorWhite);
  let mask = Bitmap::new(100, 40, SolidColor::kColorWhite);
  bmp.set_mask_bitmap(&mask).expect("mask problems");

  graphics.draw_bitmap(&bmp, 5, 9, BitmapFlip::kBitmapUnflipped);

  let mut stencil = Bitmap::new(64, 64, SolidColor::kColorWhite);
  for y in 0..64 as usize {
    let c = y % 4 != 0;
    for x in 0..64 as usize {
      stencil.as_pixels_mut().set(x, y, c.into());
    }
  }

  let font = Font::from_file("fonts/Mini Sans 2X/Mini Sans 2X.pft");
  let _active = match &font {
    Ok(font) => {
      log(format!("Font height: {}", font.font_height()));

      let page = font.font_page('d');
      log("Got page");
      let _bitmap = page.glyph('d').unwrap().bitmap();

      Some(graphics.set_font(font))
    }
    Err(e) => {
      log(format!("ERROR: loading font {}", e));
      None
    }
  };

  {
    let _stencil_holder = graphics.set_stencil(&stencil);
    graphics.draw_text("Bloop", 30, 20);
  }

  let mut copy = graphics.working_frame_bitmap();

  for y in 20..30 {
    for x in 10..20 {
      copy.as_pixels_mut().set(x, y, PixelColor::BLACK);
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
  log(format!("collision: {}", c));

  let id = graphics.push_context_bitmap(copy);
  graphics.pop_context();
  let _copy = graphics.take_popped_context_bitmap(id).unwrap();

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
    log(error);
  }

  let display = &mut api.display;
  display.set_inverted(true);
  //display.set_flipped(true, false);
  display.set_scale(2);

  let list_files_in = |path: &str| match api.file.list_files(path) {
    Ok(files) => {
      log(format!("{}/ files:", path));
      for fname in files {
        log(format!("  {:?}", fname))
      }
    }
    Err(e) => log(format!("ERROR: {}", e)),
  };
  let make_dir = |path: &str| match api.file.make_folder(path) {
    Ok(()) => log(format!("mkdir {}", path)),
    Err(e) => log(e),
  };
  let rename = |from: &str, to: &str| match api.file.rename(from, to) {
    Ok(()) => {
      log(format!("renamed {} to {}", from, to));
      list_files_in("myfolder");
    }
    Err(e) => log(e),
  };
  let delete_recursive = |path: &str| match api.file.delete_recursive(path) {
    Ok(()) => log(format!("deleted {} recursive", path)),
    Err(e) => log(e),
  };
  let stat = |path: &str| match api.file.stat(path) {
    Ok(stats) => log(format!("stat {}: {:?}", path, stats)),
    Err(e) => log(e),
  };
  let write_file = |path: &str, stuff: &[u8]| match api.file.write_file(path, stuff) {
    Ok(()) => log(format!("wrote {}", path)),
    Err(e) => log(e),
  };
  let read_file = |path: &str| match api.file.read_file(path) {
    Ok(content) => log(format!("read {}: {:?}", path, String::from_utf8(content))),
    Err(e) => log(e),
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
  log(format!("Default channel volume (in 0-1): {}", vol));

  let mut i32callbacks = Callbacks::<i32>::new();

  let mut fileplayer = FilePlayer::from_file("sounds/mojojojo.pda").unwrap();
  api.sound.default_channel_mut().add_source(&mut fileplayer).unwrap();
  fileplayer.as_mut().set_completion_callback(
    SoundCompletionCallback::with(&mut i32callbacks).call(|i| {
      log(format!("finished playback of mojojojo {}", i));
    }),
  );
  fileplayer.play(1).expect("fileplayer play failed?");
  log(format!(
    "Fileplayer length: {} seconds",
    fileplayer.file_len().to_seconds(),
  ));
  fileplayer.fade_volume(
    StereoVolume::zero(),
    TimeDelta::from_seconds(1),
    /*SoundCompletionCallback::with(&mut i32callbacks).call(|(_i, system)| {
      log("fade done!");
      log(">> getting time");
      let t = system.current_time();
      log("<< getting time");
      log(format!("time {}", t));
    }),*/
    SoundCompletionCallback::none(),
  );

  let sample = AudioSample::with_bytes(100000);
  let mut splayer = SamplePlayer::new(&sample);
  splayer.play(1, 1.0);

  struct GeneratorData {}
  let data = GeneratorData {};
  static VTABLE: SynthGeneratorVTable = SynthGeneratorVTable {
    render_func: |_data, _r| false,
    note_on_func: |_data, _note, _volume, _len| {},
    release_func: |_data, _ended| {},
    set_parameter_func: |_data, _parameter, _value| false,
  };
  let generator = unsafe { SynthGenerator::new(data, &VTABLE) };
  let mut synth = Synth::new_with_generator(generator);
  synth.play_frequency_note(0.0, 1.0.into(), None, None);
  log(format!("synth playing: {}", synth.as_source().is_playing()));

  let mut dline = DelayLine::new(TimeDelta::from_seconds(8), false);
  let tap = dline.add_tap(TimeDelta::from_seconds(6)).unwrap();
  api.sound.default_channel_mut().add_source(tap).unwrap();
  dline.set_len(TimeDelta::from_seconds(3));
  log(format!("dline length {}", dline.len()));

  let _cb_source =
    CallbackSource::new_mono_for_channel(api.sound.default_channel_mut(), |_buf: &mut [i16]| false);

  let mut sequence = Sequence::from_midi_file("sounds/pirate.mid").unwrap();
  for mut track in sequence.tracks_mut() {
    let mut instrument = track.instrument_mut();
    instrument.set_volume(StereoVolume::new(0.3, 0.3));
    api.sound.default_channel_mut().add_source(&mut instrument).unwrap();

    log(format!("polyphony: {}", track.polyphony()));
    for _ in 0..track.polyphony() {
      let mut synth = Synth::new_with_waveform(SoundWaveform::kWaveformSquare);
      synth.set_attack_time(TimeDelta::from_milliseconds(0));
      synth.set_decay_time(TimeDelta::from_milliseconds(200));
      synth.set_sustain_level(0.3);
      synth.set_release_time(TimeDelta::from_milliseconds(500));
      instrument.add_voice(synth, MidiNoteRange::All, 0.0).unwrap();
    }
  }
  sequence.play(SoundCompletionCallback::none());

  let action_item = MenuItem::new_action(
    "hello world",
    MenuCallback::with(&mut i32callbacks).call(|i| {
      log(format!("menu action {}", i));
    }),
  );
  action_item.title();
  let mut check_item = MenuItem::new_checkmark(
    "dank",
    false,
    MenuCallback::with(&mut i32callbacks).call(|i| {
      log(format!("dankness adjusted {}", i));
    }),
  );
  check_item.set_checked(true);
  let mut options_item = MenuItem::new_options(
    "temp",
    ["too hot", "too cold", "just right"],
    MenuCallback::with(&mut i32callbacks).call(|i| {
      log(format!("temperature adjusted {}", i));
    }),
  );
  options_item.set_value(2);

  log(format!(
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
        log("locked");
        continue;
      }
      SystemEvent::DidUnlock => {
        log("unlocked");
        continue;
      }
      SystemEvent::Callback => {
        i32callbacks.run(1);
        continue;
      }
      _ => continue,
    };
    for (button, event) in inputs.buttons().all_events() {
      match event {
        playdate::ButtonEvent::Push => {
          log(format!("{:?} pushed on frame {}", button, frame_number));
        }
        playdate::ButtonEvent::Release => {
          log(format!("{:?} released on frame {}", button, frame_number));
        }
      }
    }

    api.graphics.draw_fps(400 - 15, 0);
  }
}
