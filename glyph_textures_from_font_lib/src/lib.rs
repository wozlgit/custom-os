#![no_std]

use core::mem::size_of;
use core::slice;

#[repr(C)]
pub struct GlyphBitmapsHeader {
    pub num_glyphs: u16,
    pub ascent: u32,
    pub descent: u32,
    pub line_gap: u32
}

impl GlyphBitmapsHeader {
    pub fn new(num_glyphs: u16, ascent: u32, descent: u32, line_gap: u32) -> Self {
        GlyphBitmapsHeader {
            num_glyphs,
            ascent,
            descent,
            line_gap
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct GlyphBitmap {
    pub glyph: char,
    pub width_in_pixels: u32,
    pub height_in_pixels: u32,
    pub advance_width: u32,
    pub left_side_bearing: u32
}

#[derive(Clone)]
pub struct GlyphBitmapIterator<'a> {
    current_glyph: u16,
    glyph_bitmaps_bytes: &'a [u8],
    glyph_data_cache: [Option<GlyphData<'a>>; 255],
    current_offset: usize
}

impl<'a> Iterator for GlyphBitmapIterator<'a> {
    type Item = GlyphData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut data_ptr = self.glyph_bitmaps_bytes.as_ptr();
        if self.current_glyph >= self.header().num_glyphs {
            return None;
        }

        data_ptr = unsafe { data_ptr.add(self.current_offset) };
        let bitmap_header = unsafe { &*(data_ptr as *const GlyphBitmap) };
        data_ptr = unsafe { data_ptr.add(size_of::<GlyphBitmap>()) };
        let pixel_count = (bitmap_header.width_in_pixels * bitmap_header.height_in_pixels) as usize;
        let pixels = unsafe { slice::from_raw_parts(data_ptr as *const f32, pixel_count) };

        self.current_glyph += 1;
        self.current_offset =
            data_ptr as usize - self.glyph_bitmaps_bytes.as_ptr() as usize + pixel_count * size_of::<f32>();
        Some(GlyphData {
            header: bitmap_header,
            pixels
        })
    }
}

#[derive(Debug, Clone)]
pub struct GlyphData<'a> {
    pub header: &'a GlyphBitmap,
    pub pixels: &'a [f32]
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum GlyphBitmapIterError {
    AdressUnaligned = 1
}

impl<'a> GlyphBitmapIterator<'a> {
    pub fn new(glyph_bitmaps_bytes: &'a [u8]) -> Result<Self, GlyphBitmapIterError> {
        if glyph_bitmaps_bytes.as_ptr() as usize % 4 != 0 {
            return Err(GlyphBitmapIterError::AdressUnaligned);
        }
        let mut i = GlyphBitmapIterator {
            current_glyph: 0,
            glyph_bitmaps_bytes,
            glyph_data_cache: [const { None }; 255],
            current_offset: size_of::<GlyphBitmapsHeader>()
        };
        for glyph_data in i.clone() {
            i.glyph_data_cache[glyph_data.header.glyph as usize] = Some(glyph_data.clone());
        }
        Ok(i)
    }

    pub fn glyph_data(&self, glyph_char_code: char) -> Option<GlyphData<'a>> {
        if glyph_char_code <= 255 as char {
            self.glyph_data_cache[glyph_char_code as usize].clone()
        }
        else {
            self.clone()
                .find(|glyph_data| glyph_data.header.glyph == glyph_char_code)
        }
    }

    pub fn header(&self) -> &GlyphBitmapsHeader {
        unsafe { &*(self.glyph_bitmaps_bytes.as_ptr() as *const GlyphBitmapsHeader) }
    }
}

pub struct _AlignDummy<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes
}
