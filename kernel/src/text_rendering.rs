use micromath::vector::Vector2d;
use glyph_textures_from_font_lib::{GlyphData, GlyphBitmapIterator, _AlignDummy};
use crate::graphics::{Color, Rgb};
use crate::limine::LimineFramebuffer;
use spin::{Lazy, Mutex};

pub struct TextRenderer<'a> {
    text_buffer: [u8; 4096],
    pub base_pixel_offset: Vector2d<u32>,
    current_pixel_offset: Vector2d<u32>,
    glyph_bitmaps: GlyphBitmapIterator<'a>,
    pub color: Rgb,
    pub line_gap: u32,
    pub horizontal_marginal: u32,
    text_buffer_offset: usize
}

impl<'a> TextRenderer<'a> {
    pub fn add_text(&mut self, framebuffer: &mut LimineFramebuffer, text: &str) {
        let mut chars = text.chars();
        let mut c = chars.next();
        while c.is_some() {
            let chr = unsafe { c.unwrap_unchecked() };

            if chr == '\n' {
                self.new_line();
            }
            else {
                let glyph_data = self.glyph_bitmaps.glyph_data(chr).unwrap();
                // Don't wrap around if this is the first character printed
                if self.text_buffer_offset > 0 && (self.current_pixel_offset.x + glyph_data.header.width_in_pixels + self.horizontal_marginal) as u64 > framebuffer.width {
                    self.new_line();
                }
                draw_glyph_image(framebuffer, &glyph_data, &self.color, self.current_pixel_offset.x as u64, self.current_pixel_offset.y as u64);
                self.current_pixel_offset.x += glyph_data.header.advance_width;
            }

            chr.encode_utf8(&mut self.text_buffer[self.text_buffer_offset..]);
            self.text_buffer_offset += chr.len_utf8();
            c = chars.next();
        }
    }
    fn new_line(&mut self) {
        self.current_pixel_offset.x = self.base_pixel_offset.x;
        self.current_pixel_offset.y += self.line_gap;
    }
    pub fn new(text_color: Rgb, line_gap: u32, horizontal_marginal: u32, base_pixel_offset: Vector2d<u32>, glyph_bitmaps: GlyphBitmapIterator<'a>) -> TextRenderer<'a> {
        TextRenderer { 
            text_buffer: [0; 4096],
            base_pixel_offset,
            current_pixel_offset: base_pixel_offset,
            glyph_bitmaps,
            color: text_color,
            line_gap,
            horizontal_marginal,
            text_buffer_offset: 0
        }
    }
}

impl core::fmt::Write for TextRenderer<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.add_text(*crate::FRAMEBUFFER.lock(), s);
        core::fmt::Result::Ok(())
    }
}

/// Performs bound checking, and silently does nothing if drawing the glyph would result in a write
/// outside the bounds of `fb`
pub fn draw_glyph_image(fb: &mut LimineFramebuffer, glyph_data: &GlyphData, color: &Rgb, offset_x: u64, offset_y: u64) {
    if glyph_data.header.width_in_pixels as u64 + offset_x > fb.width || glyph_data.header.height_in_pixels as u64 + offset_y > fb.height {
        return;
    }
    for (index, cov) in glyph_data.pixels.iter().enumerate() {
        if *cov < 0.001 {
            continue;
        }
        let x = index % glyph_data.header.width_in_pixels as usize;
        let y = index / glyph_data.header.width_in_pixels as usize;
        let mut pixel_color = color.clone();
        pixel_color.r = ((pixel_color.r as f32) * cov) as u32;
        pixel_color.g = ((pixel_color.g as f32) * cov) as u32;
        pixel_color.b = ((pixel_color.b as f32) * cov) as u32;
        let pixel_color = Color::new(fb, pixel_color);
        unsafe { fb.set_pixel_color_unchecked(x as u64 + offset_x, y as u64 + offset_y, pixel_color); }
    }
}

static FONT_BYTES_ALIGN_DUMMY: &'static _AlignDummy<f32, [u8]> = &_AlignDummy { 
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
                crate::sleep(BLINK_INTERVAL / 2);
                crate::FRAMEBUFFER.lock().display_num(e as u8 as u32);
                crate::sleep(BLINK_INTERVAL);
            }
        }
    };
    Mutex::new(TextRenderer::new(
        Rgb { r: 255, g: 0, b: 100 },
        145,
        10,
        Vector2d { x: 0, y: 0 },
        glyph_bitmaps
    ))
});
