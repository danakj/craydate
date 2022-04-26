use alloc::format;
use core::ptr::NonNull;

use super::unowned_bitmap::UnownedBitmapRef;
use crate::capi_state::CApiState;
use crate::ctypes::*;
use crate::error::Error;
use crate::null_terminated::ToNullTerminatedString;

/// Font which can be used to draw text when made active with `Graphics::set_font()`.
#[derive(Debug)]
pub struct Font {
  font_ptr: NonNull<CFont>,
}
impl Font {
  pub(crate) fn from_ptr(font_ptr: *mut CFont) -> Self {
    Font {
      font_ptr: unsafe { NonNull::new_unchecked(font_ptr) },
    }
  }

  /// Returns the Font object for the font file at `path`.
  pub fn from_file(path: &str) -> Result<Font, Error> {
    let mut out_err: *const u8 = core::ptr::null_mut();

    // UNCLEAR: out_err is not a fixed string (it contains the name of the image). However, future
    // calls will overwrite the previous out_err and trying to free it via system->realloc crashes
    // (likely because the pointer wasn't alloc'd by us). This probably (hopefully??) means that we
    // don't need to free it.
    let font_ptr = unsafe {
      Self::fns().loadFont.unwrap()(path.to_null_terminated_utf8().as_ptr(), &mut out_err)
    };

    if !out_err.is_null() {
      let result = unsafe { crate::null_terminated::parse_null_terminated_utf8(out_err) };
      match result {
        // A valid error string.
        Ok(err) => Err(format!("load_font: {}", err).into()),
        // An invalid error string.
        Err(err) => Err(format!("load_font: unknown error ({})", err).into()),
      }
    } else {
      assert!(!font_ptr.is_null());
      Ok(Font::from_ptr(font_ptr))
    }
  }

  /// Measure the `text` string as drawn with the font.
  ///
  /// The `tracking` value is the number of pixels of whitespace between each character drawn in a
  /// string.
  pub fn measure_text_width(&self, text: &str, tracking: i32) -> i32 {
    let utf = text.to_null_terminated_utf8();
    unsafe {
      // getTextWidth() takes a mutable pointer but does not write to the data.
      Self::fns().getTextWidth.unwrap()(
        self.cptr() as *mut _,
        utf.as_ptr() as *const core::ffi::c_void,
        utf.len() as u64 - 1, // Don't count the null.
        CStringEncoding::kUTF8Encoding,
        tracking,
      )
    }
  }

  /// The height of the font.
  pub fn font_height(&self) -> u8 {
    // getFontHeight() takes a mutable pointer but does not write to the data.
    unsafe { Self::fns().getFontHeight.unwrap()(self.cptr() as *mut _) }
  }

  /// Returns the FontPage for the character `c`.
  ///
  /// Each FontPage contains information for 256 characters. All chars with the same high 24 bits
  /// share a page; specifically, if `(c1 & ~0xff) == (c2 & ~0xff)`, then c1 and c2 belong to the
  /// same page. The FontPage can be used to query information about all characters in the page.
  pub fn font_page(&self, c: char) -> FontPage {
    // getFontPage() takes a mutable pointer but does not write to the data.
    let page_ptr = unsafe { Self::fns().getFontPage.unwrap()(self.cptr() as *mut _, c as u32) };
    FontPage {
      page_ptr: unsafe { NonNull::new_unchecked(page_ptr) },
      page_test: c as u32 & 0xffffff00,
    }
  }

  pub(crate) fn cptr(&self) -> *const CFont {
    self.font_ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static craydate_sys::playdate_graphics {
    CApiState::get().cgraphics
  }
}

/// Information about a set of 256 chars.
///
/// All chars with the same high 24 bits share a page; specifically, if `(c1 & ~0xff) == (c2 &
/// ~0xff)`, then c1 and c2 belong to the same page. The FontPage can be used to query information
/// about all characters in the page.
pub struct FontPage {
  page_ptr: NonNull<CFontPage>,
  /// If a characters high 24 bits match this, then it's part of the page.
  page_test: u32,
}
impl FontPage {
  /// Whether the FontPage contains information for the character `c`.
  ///
  /// Each FontPage contains information for 256 characters. All chars with the same high 24 bits
  /// share a page; specifically, if `(c1 & ~0xff) == (c2 & ~0xff)`, then c1 and c2 belong to the
  /// same page.
  pub fn contains(&self, c: char) -> bool {
    c as u32 & 0xffffff00 == self.page_test
  }

  /// Returns the glyph for the character `c`.
  ///
  /// May return None if the character is not part of this FontPage. Each FontPage contains
  /// information for 256 characters. All chars with the same high 24 bits share a page;
  /// specifically, if `(c1 & ~0xff) == (c2 & ~0xff)`, then c1 and c2 belong to the same page.
  pub fn glyph(&self, c: char) -> Option<FontGlyph> {
    if !self.contains(c) {
      None
    } else {
      // UNCLEAR: getPageGlyph says the `bitmap_ptr` and `advance` are optional but passing null
      // for either one crashes.
      let mut bitmap_ptr: *mut CBitmap = core::ptr::null_mut();
      let mut advance = 0;
      let glyph_ptr = unsafe {
        // getPageGlyph() takes a mutable pointer but does not write to the data.
        Self::fns().getPageGlyph.unwrap()(
          self.cptr() as *mut _,
          c as u32,
          &mut bitmap_ptr,
          &mut advance,
        )
      };
      Some(FontGlyph {
        glyph_ptr: NonNull::new(glyph_ptr).unwrap(),
        advance,
        glyph_char: c,
        bitmap: UnownedBitmapRef::<'static>::from_ptr(NonNull::new(bitmap_ptr).unwrap()),
      })
    }
  }

  pub(crate) fn cptr(&self) -> *const CFontPage {
    self.page_ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static craydate_sys::playdate_graphics {
    CApiState::get().cgraphics
  }
}

/// Information about a specific character's font glyph.
pub struct FontGlyph {
  glyph_ptr: NonNull<CFontGlyph>,
  advance: i32,
  glyph_char: char,
  // Fonts can not be unloaded/destroyed, so the bitmap has a static lifetime.
  bitmap: UnownedBitmapRef<'static>,
}
impl FontGlyph {
  /// Returns the advance value for the glyph, which is the width that should be allocated for the
  /// glyph.
  pub fn advance(&self) -> i32 {
    self.advance
  }

  /// Returns the kerning adjustment between the glyph and `next_char` as specified by the font.
  ///
  /// The adjustment would be applied to the `advance()`.
  pub fn kerning(&self, next_char: char) -> i32 {
    unsafe {
      // getGlyphKerning() takes a mutable pointer but does not write to the data.
      Self::fns().getGlyphKerning.unwrap()(
        self.cptr() as *mut _,
        self.glyph_char as u32,
        next_char as u32,
      )
    }
  }

  /// The bitmap representation of the font glyph.
  pub fn bitmap(&self) -> UnownedBitmapRef<'static> {
    self.bitmap.clone()
  }

  pub(crate) fn cptr(&self) -> *const CFontGlyph {
    self.glyph_ptr.as_ptr()
  }
  pub(crate) fn fns() -> &'static craydate_sys::playdate_graphics {
    CApiState::get().cgraphics
  }
}
