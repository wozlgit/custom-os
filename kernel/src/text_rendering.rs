use core::cmp::min;
use core::str::from_utf8_unchecked;

use glyph_textures_from_font_lib::{GlyphData, GlyphBitmapIterator, _AlignDummy};
use crate::graphics::{Color, Rgb};
use crate::limine::LimineFramebuffer;
use spin::{Lazy, Mutex};

#[derive(Clone, Copy)]
pub struct Vec2<T> {
    x: T,
    y: T
}

pub struct TextRenderer<'a> {
    text_buffer: [u8; 4096],
    text_buffer_offset: usize,
    current_pixel_offset: Vec2<i64>,
    settings: TextRendererSettings<'a>
}

impl<'a> TextRenderer<'a> {
    pub fn add_text(&mut self, framebuffer: &mut LimineFramebuffer, text: &str) {
        for c in text.chars() {
            c.encode_utf8(&mut self.text_buffer[self.text_buffer_offset..]);
            self.text_buffer_offset += c.len_utf8();
            self.current_pixel_offset = draw_character(c, self.current_pixel_offset, &mut self.settings, &self.text_buffer, self.text_buffer_offset, framebuffer);
        }
    }
    pub fn new(mut settings: TextRendererSettings<'a>) -> TextRenderer<'a> {
        settings.base_pixel_offset.y += settings.glyph_bitmaps.header().ascent as i64;
        TextRenderer { 
            text_buffer: [0; 4096],
            text_buffer_offset: 0,
            current_pixel_offset: settings.base_pixel_offset,
            settings
        }
    }
}

pub struct TextRendererSettings<'a> {
    pub base_pixel_offset: Vec2<i64>,
    pub vertical_marginal: u32,
    pub horizontal_marginal: u32,
    pub line_gap: u32,
    pub text_color: Rgb, 
    pub glyph_bitmaps: GlyphBitmapIterator<'a>
}

impl core::fmt::Write for TextRenderer<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.add_text(*crate::FRAMEBUFFER.lock(), s);
        core::fmt::Result::Ok(())
    }
}

fn draw_character(
    c: char,
    mut current_pixel_offset: Vec2<i64>,
    settings: &mut TextRendererSettings,
    text_buffer: &[u8], text_buffer_offset: usize,
    framebuffer: &mut LimineFramebuffer
) -> Vec2<i64> {
    if c == '\n' {
        current_pixel_offset = new_line(current_pixel_offset, text_buffer, text_buffer_offset, settings, framebuffer);
    }
    else {
        let glyph_data = settings.glyph_bitmaps.glyph_data(c).unwrap();
        // Don't wrap around if this is the first character on this line
        if current_pixel_offset.x > settings.base_pixel_offset.x &&
          current_pixel_offset.x + (glyph_data.header.width_in_pixels + settings.horizontal_marginal) as i64 > framebuffer.width as i64
        {
            current_pixel_offset = new_line(current_pixel_offset, text_buffer, text_buffer_offset, settings, framebuffer);
        }
        if glyph_data.header.width_in_pixels > 0 && glyph_data.header.height_in_pixels > 0 {
            let offset_x = current_pixel_offset.x + glyph_data.header.left_side_bearing as i64;
            let offset_y = current_pixel_offset.y - min(glyph_data.header.height_in_pixels - 1, settings.glyph_bitmaps.header().ascent) as i64;
            draw_glyph_image(framebuffer, &glyph_data, &settings.text_color, offset_x, offset_y);
        }
        current_pixel_offset.x += glyph_data.header.advance_width as i64;
    }
    current_pixel_offset
}

fn new_line(
    current_pixel_offset: Vec2<i64>,
    text_buffer: &[u8], text_buffer_offset: usize,
    settings: &mut TextRendererSettings,
    framebuffer: &mut LimineFramebuffer
) -> Vec2<i64> {
    let mut new_baseline = Vec2 {
        x: settings.base_pixel_offset.x,
        y: current_pixel_offset.y + settings.line_gap as i64
    };

    let required_space_y = (settings.vertical_marginal + settings.glyph_bitmaps.header().descent) as i64;
    let space_y = framebuffer.height as i64 - new_baseline.y;
    let more_space_required_y = required_space_y - space_y;
    if more_space_required_y > 0 {
        let bg_color = Color::new(framebuffer, Rgb { r: 100, g: 255, b: 0 });
        framebuffer.fill(bg_color);
        settings.base_pixel_offset.y -= more_space_required_y;
        new_baseline = settings.base_pixel_offset;
        // Guaranteed to be valid UTF8, since only this struct can write to the text buffer. Only
        // the bytes which have been set to some character are included here, though the buffer is
        // zero initialized anyway (zero is valid UTF-8).
        let text = unsafe { from_utf8_unchecked(&text_buffer[..text_buffer_offset]) };
        // Now draw every character on the text buffer as normal. It is impossible that this piece of code would be called
        // again in the process, since enough space has been "allocated" such that every single
        // pixel of all characters at the end of the text buffer fits on screen, and only some
        // characters at the start of the text buffer will have pixels that don't fit on
        // screen.
        for c in text.chars() {
            new_baseline = draw_character(c, new_baseline, settings, text_buffer, text_buffer_offset, framebuffer);
        }
    }
    new_baseline
}

/// Draws the glyph into `fb`, silently ignoring every pixel of the glyph that has an offset
/// outside the bounds of `fb`. Negative values can be passed as `offset_x` and `offset_y` to
/// partially render the glyph, drawing every pixel that ends up having an offset inside the
/// framebuffer.
pub fn draw_glyph_image(fb: &mut LimineFramebuffer, glyph_data: &GlyphData, color: &Rgb, offset_x: i64, offset_y: i64) {
    for (index, cov) in glyph_data.pixels.iter().enumerate() {
        if *cov < 0.001 {
            continue;
        }
        let x = index as i64 % glyph_data.header.width_in_pixels as i64;
        let y = index as i64 / glyph_data.header.width_in_pixels as i64;
        let pixel_pos_x = x + offset_x;
        let pixel_pos_y = y + offset_y;
        if pixel_pos_x < 0 || pixel_pos_y < 0 {
            continue;
        }
        if pixel_pos_x as u64 >= fb.width || pixel_pos_y as u64 >= fb.height {
            continue;
        }
        let mut pixel_color = color.clone();
        pixel_color.r = ((pixel_color.r as f32) * cov) as u32;
        pixel_color.g = ((pixel_color.g as f32) * cov) as u32;
        pixel_color.b = ((pixel_color.b as f32) * cov) as u32;
        let pixel_color = Color::new(fb, pixel_color);
        unsafe { fb.set_pixel_color_unchecked(pixel_pos_x as u64, pixel_pos_y as u64, pixel_color) };
    }
}

pub static FONT_BYTES_ALIGN_DUMMY: &'static _AlignDummy<f32, [u8]> = &_AlignDummy { 
    _align: [],
    bytes: *include_bytes!("../../glyph_textures_from_font/glyph_bitmaps.bin")
};

pub static TEXT_RENDERER: Lazy<Mutex<TextRenderer>> = Lazy::new(|| {
    let glyph_bitmaps = match GlyphBitmapIterator::new(&FONT_BYTES_ALIGN_DUMMY.bytes) {
        Ok(g) => g,
        Err(e) => {
            let color = Color::new(*crate::FRAMEBUFFER.lock(), Rgb { r: 0, g: 0, b: 0 });
            const BLINK_INTERVAL: u64 = 90000000;
            loop {
                crate::FRAMEBUFFER.lock().fill(color);
                crate::sleep(BLINK_INTERVAL / 4);
                crate::FRAMEBUFFER.lock().display_num(e as u8 as u32);
                crate::sleep(BLINK_INTERVAL);
            }
        }
    };
    let settings = TextRendererSettings {
        text_color: Rgb { r: 255, g: 0, b: 100 },
        line_gap: glyph_bitmaps.header().line_gap,
        horizontal_marginal: 10,
        vertical_marginal: 50,
        base_pixel_offset: Vec2 { x: 0, y: 0 },
        glyph_bitmaps
    };
    Mutex::new(TextRenderer::new(settings))
});

#[macro_export]
macro_rules! print {
    ($($e:expr),*) => { 
        {
            use core::fmt::Write;
            core::write!(*crate::text_rendering::TEXT_RENDERER.lock(), $($e),*).unwrap()
        }
    }
}

#[macro_export]
macro_rules! println {
    ($($e:expr),*) => { 
        {
            use core::fmt::Write;
            core::writeln!(*crate::text_rendering::TEXT_RENDERER.lock(), $($e),*).unwrap()
        }
    }
}
