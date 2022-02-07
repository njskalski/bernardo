use std::fmt::Formatter;

use hex::FromHexError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{EnumAccess, Error, MapAccess, SeqAccess, Visitor};

#[derive(Clone, Copy, Eq, PartialOrd, PartialEq, Hash, Debug)]
pub struct Color {
    pub R: u8,
    pub G: u8,
    pub B: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { R: r, G: g, B: b }
    }

    pub fn half(&self) -> Self {
        Color { R: self.R / 2, G: self.R / 2, B: self.B / 2 }
    }

    pub fn interpolate(a: Color, b: Color) -> Color {
        let r: u16 = (a.R as u16 + b.R as u16) / 2;
        let g: u16 = (a.B as u16 + b.B as u16) / 2;
        let b: u16 = (a.G as u16 + b.G as u16) / 2;
        Color {
            R: r as u8,
            G: g as u8,
            B: b as u8,
        }
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&format!("#{:02X}{:02X}{:02X}", self.R, self.G, self.B))
    }
}

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a color written in \"#(r)(g)(b)\" format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        if v.len() != 7 {
            return Err(E::custom(format!("length should be 7 and is {}", v.len())));
        }

        if v.chars().next() != Some('#') {
            return Err(E::custom(format!("expected first character to be \"#\", got \"{}\"", v.chars().next().unwrap())));
        }

        let mut decoded: [u8; 3] = [0; 3];
        match hex::decode_to_slice(&v[1..], &mut decoded) {
            Ok(()) => {
                Ok(Color {
                    R: decoded[0],
                    G: decoded[1],
                    B: decoded[2],
                })
            },
            Err(e) => Err(E::custom(format!("failed hex decoding: {:?}", e))),
        }
    }
}

impl<'a> Deserialize<'a> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
        deserializer.deserialize_str(ColorVisitor)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        assert_eq!(ron::to_string(&Color::new(0, 0, 0)), Ok("\"#000000\"".to_string()));
        assert_eq!(ron::to_string(&Color::new(0, 100, 0)), Ok("\"#006400\"".to_string()));
        assert_eq!(ron::to_string(&Color::new(0, 255, 16)), Ok("\"#00FF10\"".to_string()));
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(ron::from_str("\"#00FF0F\""), Ok(Color::new(0, 255, 15)));
        assert_eq!(ron::from_str("\"#006400\""), Ok(Color::new(0, 100, 0)));
    }
}