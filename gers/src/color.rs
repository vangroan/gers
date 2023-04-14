use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[allow(dead_code)]
    pub fn from_hex(_hex: &str) -> Self {
        todo!("parse color from hex code")
    }

    pub fn as_f32(&self) -> [f32; 4] {
        [
            self.r as f32 / u8::MAX as f32,
            self.g as f32 / u8::MAX as f32,
            self.b as f32 / u8::MAX as f32,
            self.a as f32 / u8::MAX as f32,
        ]
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { r, g, b, a } = self;
        write!(f, "#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_color_size() {
        debug_assert_eq!(std::mem::size_of::<Color>(), std::mem::size_of::<u32>());
    }
}
