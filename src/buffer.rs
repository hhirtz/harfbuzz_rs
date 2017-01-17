use hb;
use std;

use font::Font;

pub type GlyphPosition = hb::hb_glyph_position_t;
pub type GlyphInfo = hb::hb_glyph_info_t;
pub type Feature = hb::hb_feature_t;

fn language_to_string(tag: hb::hb_language_t) -> &'static str {
    let lang_string_ptr = unsafe { hb::hb_language_to_string(tag) };
    let cstring = unsafe { std::ffi::CStr::from_ptr(lang_string_ptr) };
    cstring.to_str().expect("harfbuzz error: language string is not valid utf-8!")
}

struct BufferRaw {
    hb_buffer: *mut hb::hb_buffer_t,
}
impl BufferRaw {
    fn new() -> BufferRaw {
        let buffer = unsafe { hb::hb_buffer_create() };

        BufferRaw { hb_buffer: buffer }
    }

    fn as_raw(&self) -> *mut hb::hb_buffer_t {
        self.hb_buffer
    }

    fn len(&self) -> usize {
        unsafe { hb::hb_buffer_get_length(self.hb_buffer) as usize }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn add_str(&mut self, string: &str) {
        let utf8_ptr = string.as_ptr() as *const i8;
        unsafe {
            hb::hb_buffer_add_utf8(self.hb_buffer,
                                   utf8_ptr,
                                   string.len() as i32,
                                   0,
                                   string.len() as i32);
        }
    }

    fn set_direction(&mut self, direction: hb::hb_direction_t) {
        unsafe { hb::hb_buffer_set_direction(self.hb_buffer, direction) };
    }

    /// Returns the `Buffer`'s text direction.
    fn get_direction(&self) -> hb::hb_direction_t {
        unsafe { hb::hb_buffer_get_direction(self.hb_buffer) }
    }

    fn get_language(&self) -> &'static str {
        let lang = unsafe { hb::hb_buffer_get_language(self.hb_buffer) };
        language_to_string(lang)
    }

    fn guess_segment_properties(&mut self) {
        unsafe { hb::hb_buffer_guess_segment_properties(self.hb_buffer) };
    }

    fn get_segment_properties(&self) -> hb::hb_segment_properties_t {
        unsafe {
            let mut segment_props: hb::hb_segment_properties_t = std::mem::uninitialized();
            hb::hb_buffer_get_segment_properties(self.hb_buffer, &mut segment_props as *mut _);
            segment_props
        }
    }

    fn shape(&mut self, font: &Font, features: &[Feature]) {
        unsafe {
            hb::hb_shape(font.as_raw(),
                         self.hb_buffer,
                         features.as_ptr(),
                         features.len() as u32)
        };
    }

    fn clear_contents(&mut self) {
        unsafe { hb::hb_buffer_clear_contents(self.hb_buffer) };
    }

    fn get_glyph_positions(&self) -> &mut [GlyphPosition] {
        unsafe {
            let mut length: u32 = 0;
            let glyph_pos = hb::hb_buffer_get_glyph_positions(self.hb_buffer,
                                                              &mut length as *mut u32);
            std::slice::from_raw_parts_mut(glyph_pos, length as usize)
        }
    }

    fn get_glyph_infos(&self) -> &mut [GlyphInfo] {
        unsafe {
            let mut length: u32 = 0;
            let glyph_infos = hb::hb_buffer_get_glyph_infos(self.hb_buffer,
                                                            &mut length as *mut u32);
            std::slice::from_raw_parts_mut(glyph_infos, length as usize)
        }
    }

    /// Reverse the `Buffer`'s contents.
    fn reverse(&mut self) {
        unsafe { hb::hb_buffer_reverse(self.hb_buffer) };
    }

    /// Reverse the `Buffer`'s contents in the range from `start` to `end`.
    fn reverse_range(&mut self, start: usize, end: usize) {
        assert!(start <= self.len(), end <= self.len());
        unsafe {
            hb::hb_buffer_reverse_range(self.hb_buffer, start as u32, end as u32)
        }
    }
}

impl Drop for BufferRaw {
    fn drop(&mut self) {
        unsafe {
            hb::hb_buffer_destroy(self.hb_buffer);
        }
    }
}

/// A `UnicodeBuffer` can be filled with unicode text and corresponding
pub struct UnicodeBuffer(BufferRaw);
#[allow(dead_code)]
impl UnicodeBuffer {
    /// Creates a new empty `Buffer`.
    pub fn new() -> UnicodeBuffer {
        UnicodeBuffer(BufferRaw::new())
    }

    /// Returns a pointer to the underlying raw harfbuzz buffer.
    pub fn as_raw(&self) -> *mut hb::hb_buffer_t {
        self.0.as_raw()
    }

    /// Returns the length of the data of the buffer.
    ///
    /// When called before shaping this is the number of unicode codepoints contained in the
    /// buffer. When called after shaping it returns the number of glyphs stored.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Adds the string slice to the `Buffer`'s array of codepoints.
    pub fn add_str(mut self, string: &str) -> UnicodeBuffer {
        self.0.add_str(string);
        self
    }

    /// Sets the text direction of the `Buffer`'s contents.
    pub fn set_direction(mut self, direction: hb::hb_direction_t) -> UnicodeBuffer {
        self.0.set_direction(direction);
        self
    }

    /// Returns the `Buffer`'s text direction.
    pub fn get_direction(&self) -> hb::hb_direction_t {
        self.0.get_direction()
    }

    pub fn get_language(&self) -> &'static str {
        self.0.get_language()
    }

    pub fn guess_segment_properties(mut self) -> UnicodeBuffer {
        self.0.guess_segment_properties();
        self
    }

    pub fn get_segment_properties(&self) -> hb::hb_segment_properties_t {
        self.0.get_segment_properties()
    }

    pub fn shape(mut self, font: &Font, features: &[Feature]) -> GlyphBuffer {
        self = self.guess_segment_properties();
        self.0.shape(font, features);
        GlyphBuffer(self.0)
    }

    pub fn clear_contents(mut self) -> UnicodeBuffer {
        self.0.clear_contents();
        self
    }
}

impl std::fmt::Debug for UnicodeBuffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Buffer")
            .field("len", &self.len())
            .field("direction", &self.get_direction())
            .field("language", &self.get_language().to_owned())
            .finish()
    }
}

impl std::default::Default for UnicodeBuffer {
    fn default() -> UnicodeBuffer {
        UnicodeBuffer::new()
    }
}

/// A `GlyphBuffer` is obtained through the `shape` function of a `UnicodeBuffer`. It contains
/// the resulting output information of the shaping process.
pub struct GlyphBuffer(BufferRaw);

impl GlyphBuffer {
    /// Returns the length of the data of the buffer.
    ///
    /// When called before shaping this is the number of unicode codepoints contained in the
    /// buffer. When called after shaping it returns the number of glyphs stored.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_glyph_positions(&self) -> &mut [GlyphPosition] {
        self.0.get_glyph_positions()
    }

    pub fn get_glyph_infos(&self) -> &mut [GlyphInfo] {
        self.0.get_glyph_infos()
    }

    /// Reverse the `Buffer`'s contents.
    pub fn reverse(&mut self) {
        self.0.reverse()
    }

    /// Reverse the `Buffer`'s contents in the range from `start` to `end`.
    pub fn reverse_range(&mut self, start: usize, end: usize) {
        self.0.reverse_range(start, end)
    }

    /// Clears the contents of the glyph buffer and returns an empty `UnicodeBuffer` reusing the
    /// existing allocation.
    pub fn clear(mut self) -> UnicodeBuffer {
        self.0.clear_contents();
        UnicodeBuffer(self.0)
    }
}