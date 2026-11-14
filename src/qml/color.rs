//! Colours, on their way to QML.

use cxx_qt_lib::QColor;

use crate::color::Color;

/// The QColor of a colour.
pub fn qcolor(color: Color) -> QColor {
    QColor::from_rgba_f(color.r as f32, color.g as f32, color.b as f32, color.a as f32)
}
