use std::{fs::File, io::Write, slice::from_raw_parts};
use rusttype::{Font, Scale, Point, IntoGlyphId};

use glyph_textures_from_font_lib::*;

fn main() {
    let font_bytes = include_bytes!("../font.ttf");
    let font = Font::try_from_bytes(font_bytes).unwrap();
    let mut glyph_bitmaps: Vec<(GlyphBitmap, Vec<f32>)> = Vec::new();
    let font_scale = Scale { x: 65.0, y: 65.0 };
    for c in (0 as char)..(255 as char) {
        if c < ' ' || c > '~' {
            continue;
        }
        let glyph = font.glyph(c.into_glyph_id(&font));
        let glyph = glyph.scaled(font_scale);
        let h_metrics = glyph.h_metrics();
        let glyph = glyph.positioned(Point { x: 0.0, y: 0.0 });

        let mut bitmap = GlyphBitmap {
            glyph: c,
            width_in_pixels: 0,
            height_in_pixels: 0,
            advance_width: h_metrics.advance_width as u32,
            left_side_bearing: h_metrics.left_side_bearing as u32
        };
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            bitmap.width_in_pixels = bounding_box.width() as u32;
            bitmap.height_in_pixels = bounding_box.height() as u32;
        }
        else {
            println!("No bounding box for char: \'{}\' ({})", c, c as u32);
        }

        let mut pixels: Vec<f32> = Vec::with_capacity((bitmap.width_in_pixels * bitmap.height_in_pixels) as usize);
        let rasterize_cb = |_x: u32, _y: u32, coverage: f32| pixels.push(coverage);
        glyph.draw(rasterize_cb);

        glyph_bitmaps.push((bitmap, pixels));
    }
    let v_metrics = font.v_metrics(font_scale);
    // Apparently some fonts provide a value in v_metrics (for some `Scale`s atleast) for line_gap,
    // and some dont. So if a value is provided there, it can be used here.
    let header = GlyphBitmapsHeader::new(glyph_bitmaps.len() as u16, v_metrics.ascent as u32, v_metrics.descent as u32, 50);
    let mut file = File::create("glyph_bitmaps.bin").expect("could not create file");
    file.write(struct_byte_representation(&header)).unwrap();
    for (bitmap, cov_vec) in glyph_bitmaps.into_iter() {
        file.write(struct_byte_representation(&bitmap)).unwrap();
        unsafe {
            file.write(from_raw_parts(cov_vec.as_ptr() as *const u8, cov_vec.len() * core::mem::size_of::<f32>())).unwrap();
        }
    }


    // IMPORTANT! The font_bytes here contains the bytes of the file generated in the PREVIOUS
    // invocation of this tool, not the current one!
    let font_bytes = include_bytes!("../glyph_bitmaps.bin");
    let glyphs_iter = GlyphBitmapIterator::new(font_bytes).unwrap();
    println!("Ascent: {}, line gap: {}", glyphs_iter.header().ascent, glyphs_iter.header().line_gap);
    for glyph_data in glyphs_iter {
        println!("{}, {}, {}", glyph_data.header.width_in_pixels, glyph_data.header.height_in_pixels, glyph_data.header.left_side_bearing);
        println!("Glyph: {} ({})", glyph_data.header.glyph, glyph_data.header.glyph as u32);
    }
}

fn struct_byte_representation<T: Sized>(s: &T) -> &[u8] {
    unsafe { from_raw_parts(s as *const T as *const u8, core::mem::size_of::<T>()) }
}
