//! A colour as the rules work with it: four numbers from 0 to 1. The QML
//! types turn it into a QColor where it meets Qt.

use crate::easing::clamp01;

/// Red, green, blue and alpha, each from 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    /// An opaque colour from components, each clamped to 0..1.
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// A colour from components, each clamped to 0..1.
    pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            r: clamp01(r),
            g: clamp01(g),
            b: clamp01(b),
            a: clamp01(a),
        }
    }

    /// An opaque colour from "#rrggbb"; `None` for anything else.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let digits = hex.strip_prefix('#')?;
        if digits.len() != 6 || !digits.is_ascii() {
            return None;
        }
        let part = |i: usize| {
            u8::from_str_radix(&digits[i..i + 2], 16)
                .ok()
                .map(|v| f64::from(v) / 255.0)
        };
        Some(Self::rgb(part(0)?, part(2)?, part(4)?))
    }

    /// A colour written in the source, which must be "#rrggbb".
    pub fn hex(hex: &str) -> Self {
        Self::from_hex(hex).unwrap_or_else(|| panic!("not a colour: {hex}"))
    }

    /// From this colour toward `other` by `t`, opaque.
    pub fn mix(self, other: Self, t: f64) -> Self {
        let towards = |from: f64, to: f64| from + (to - from) * t;
        Self::rgb(
            towards(self.r, other.r),
            towards(self.g, other.g),
            towards(self.b, other.b),
        )
    }

    /// The same colour with another alpha.
    pub fn with_alpha(self, a: f64) -> Self {
        Self::rgba(self.r, self.g, self.b, a)
    }

    /// How light the colour is, 0 for black and 1 for white (HSL lightness).
    #[cfg(test)]
    pub fn lightness(self) -> f64 {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        (max + min) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colours_are_read() {
        assert_eq!(Color::from_hex("#ff0080"), Some(Color::rgb(1.0, 0.0, 128.0 / 255.0)));
        assert_eq!(Color::from_hex("ff0080"), None);
        assert_eq!(Color::from_hex("#ff00"), None);
        assert_eq!(Color::from_hex("#gg0000"), None);
    }

    #[test]
    fn mixing_goes_from_one_colour_to_the_other() {
        let (black, white) = (Color::rgb(0.0, 0.0, 0.0), Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(black.mix(white, 0.0), black);
        assert_eq!(black.mix(white, 1.0), white);
        assert_eq!(black.mix(white, 0.5).lightness(), 0.5);
    }
}
