use std::fmt::Formatter;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Eq, PartialOrd, PartialEq, Hash, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub fn half(&self) -> Self {
        Color {
            r: self.r / 2,
            g: self.r / 2,
            b: self.b / 2,
        }
    }

    pub fn interpolate(a: Color, b: Color) -> Color {
        let r: u16 = (a.r as u16 + b.r as u16) / 2;
        let g: u16 = (a.b as u16 + b.b as u16) / 2;
        let b: u16 = (a.g as u16 + b.g as u16) / 2;
        Color {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        }
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b))
    }
}

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a color written in \"#(r)(g)(b)\" format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v.len() != 7 {
            return Err(E::custom(format!("length should be 7 and is {}", v.len())));
        }

        if v.chars().next() != Some('#') {
            return Err(E::custom(format!(
                "expected first character to be \"#\", got \"{}\"",
                v.chars().next().unwrap()
            )));
        }

        let mut decoded: [u8; 3] = [0; 3];
        match hex::decode_to_slice(&v[1..], &mut decoded) {
            Ok(()) => Ok(Color {
                r: decoded[0],
                g: decoded[1],
                b: decoded[2],
            }),
            Err(e) => Err(E::custom(format!("failed hex decoding: {:?}", e))),
        }
    }
}

impl<'a> Deserialize<'a> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(tuple: (u8, u8, u8)) -> Self {
        Color {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
        }
    }
}

pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };

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
