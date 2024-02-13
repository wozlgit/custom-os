use super::limine::LimineFramebuffer;

#[derive(Clone)]
pub struct Rgb {
    pub r: u32,
    pub g: u32,
    pub b: u32
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Color(u32);

#[repr(u8)]
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum ColorError {
    ColorTooLarge
}

impl Color {
    #[inline]
    pub fn new(framebuffer: &LimineFramebuffer, color_values: Rgb) -> Color {
        Self::new_closest(framebuffer, color_values)
    }
    /// Returns a new `Color` with `color_values` clamped to their max size.
    pub fn new_closest(framebuffer: &LimineFramebuffer, mut color_values: Rgb) -> Color {
        // *_mask_size and *_mask_shift are both in bits, not bytes (haven't found any
        // documentation on this, but verified via experimentation)
        let red_max_size = (1u32 << framebuffer.red_mask_size) - 1;
        let green_max_size = (1u32 << framebuffer.green_mask_size) - 1;
        let blue_max_size = (1u32 << framebuffer.blue_mask_size) - 1;
        if color_values.r > red_max_size {
            color_values.r = red_max_size;
        }
        if color_values.g > green_max_size {
            color_values.g = green_max_size;
        }
        if color_values.b > blue_max_size {
            color_values.b = blue_max_size;
        }
        Self::color_from_individual_bound_checked_consituents(framebuffer, color_values)
    }
    /// Returns a new `Color` or `ColorError::ColorTooLarge` if `color_values` are too large
    pub fn new_if_within_bounds(framebuffer: &LimineFramebuffer, color_values: Rgb) -> Result<Color, ColorError> {
        let red_max_size = (1u32 << framebuffer.red_mask_size) - 1;
        let green_max_size = (1u32 << framebuffer.green_mask_size) - 1;
        let blue_max_size = (1u32 << framebuffer.blue_mask_size) - 1;
        if color_values.r > red_max_size {
            return Result::Err(ColorError::ColorTooLarge);
        }
        if color_values.g > green_max_size {
            return Result::Err(ColorError::ColorTooLarge);
        }
        if color_values.b > blue_max_size {
            return Result::Err(ColorError::ColorTooLarge);
        }
        Result::Ok(Self::color_from_individual_bound_checked_consituents(framebuffer, color_values))
    }
    #[inline]
    fn color_from_individual_bound_checked_consituents(framebuffer: &LimineFramebuffer, constituents: Rgb) -> Color {
        Color(
            0u32
            | (constituents.r << framebuffer.red_mask_shift)
            | (constituents.g << framebuffer.green_mask_shift)
            | (constituents.b << framebuffer.blue_mask_shift)
        )
    }
}

impl LimineFramebuffer {
    #[inline]
    pub fn get_pixel_offset(&self, x: u64, y: u64) -> u64 {
        y * (self.pitch / (self.bpp as u64 / 8)) + x
    }
    #[inline]
    pub fn set_pixel_color(&self, x: u64, y: u64, color: Color) {
        if x > self.width || y > self.height {
            // ERROR
        }
        else {
            self._set_pixel_color(x, y, color);
        }
    }
    #[inline]
    pub unsafe fn set_pixel_color_unchecked(&self, x: u64, y: u64, color: Color) {
        self._set_pixel_color(x, y, color);
    }
    #[inline]
    fn _set_pixel_color(&self, x: u64, y: u64, color: Color) {
        let bytes_per_pixel = (self.bpp / 8) as u64;
        let color_p = (&color as *const Color) as *const u8;
        let pixel_base_ptr: *mut u8 = unsafe { (self.address as *mut u8).add((self.get_pixel_offset(x, y) * bytes_per_pixel) as usize) };
        unsafe {
            for i in 0..bytes_per_pixel {
                pixel_base_ptr.offset(i as isize).write_volatile(
                    *color_p.offset(i as isize)
                );
            }
        }
    }
    /// Displays a number on the screen, by drawing that many boxes. This should only be used for
    /// debugging purposes, when text rendering is not available or functioning.
    pub fn display_num(&mut self, num: u32) {
        let color = Color::new(self, Rgb { r: 0xff, g: 0x00, b: 0x00 });
        let squares_per_row = 10;
        let pixel_padding_x = self.width / squares_per_row;
        let pixel_padding_y = self.height / 10;
        for i in 0..num {
            let x = i as u64 % squares_per_row;
            let y = i as u64 / squares_per_row;
            let pixel_offset_x = pixel_padding_x * x;
            let pixel_offset_y = pixel_padding_y * y;
            for x in 0..50 {
                for y in 0..50 {
                    unsafe { self.set_pixel_color_unchecked(x + pixel_offset_x, y + pixel_offset_y, color); }
                }
            }
        }
    }
    #[inline]
    pub fn fill(&mut self, color: Color) {
        for x in 0..self.width {
            for y in 0..self.height {
                unsafe { self.set_pixel_color_unchecked(x, y, color); }
            }
        }
    }
}
