#[derive(Clone, Copy, Eq, PartialOrd, PartialEq, Hash, Debug)]
pub struct Color {
    pub R: u8,
    pub G: u8,
    pub B: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { R: r, G: g, B: b }
    }

    pub fn half(&self) -> Self {
        Color { R: self.R / 2, G: self.R / 2, B: self.B / 2 }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(tuple: (u8, u8, u8)) -> Self {
        Color {
            R: tuple.0,
            G: tuple.1,
            B: tuple.2,
        }
    }
}

pub const BLACK: Color = Color { R: 0, G: 0, B: 0 };
pub const WHITE: Color = Color { R: 255, G: 255, B: 255 };


