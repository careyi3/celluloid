use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// An RGB colour, written as `"#rrggbb"` in animation files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub [u8; 3]);

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b])
    }

    /// Parse `#rrggbb` or `#rgb`.
    pub fn hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);
        let digit = |i: usize, len: usize| u8::from_str_radix(s.get(i..i + len)?, 16).ok();
        match s.len() {
            6 => Some(Self([digit(0, 2)?, digit(2, 2)?, digit(4, 2)?])),
            3 => Some(Self([digit(0, 1)? * 17, digit(1, 1)? * 17, digit(2, 1)? * 17])),
            _ => None,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b] = self.0;
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

impl FromStr for Color {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::hex(s).ok_or_else(|| crate::Error(format!("invalid colour {s:?}, expected #rrggbb")))
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Colours handed out to states and markers that are used without being
/// given one. Picked to stay distinct on a dark background.
pub(crate) const PALETTE: [Color; 10] = [
    Color::rgb(0x4e, 0x79, 0xa7),
    Color::rgb(0xf2, 0x8e, 0x2b),
    Color::rgb(0x59, 0xa1, 0x4f),
    Color::rgb(0xe1, 0x57, 0x59),
    Color::rgb(0xed, 0xc9, 0x48),
    Color::rgb(0xb0, 0x7a, 0xa1),
    Color::rgb(0x76, 0xb7, 0xb2),
    Color::rgb(0xff, 0x9d, 0xa7),
    Color::rgb(0x9c, 0x75, 0x5f),
    Color::rgb(0xba, 0xb0, 0xac),
];

pub(crate) const EMPTY: Color = Color::rgb(0x2a, 0x2c, 0x31);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip() {
        assert_eq!(Color::hex("#0a0B0c"), Some(Color::rgb(10, 11, 12)));
        assert_eq!(Color::hex("#fff"), Some(Color::rgb(255, 255, 255)));
        assert_eq!(Color::hex("#ff"), None);
        assert_eq!(Color::rgb(1, 2, 255).to_string(), "#0102ff");
    }
}
