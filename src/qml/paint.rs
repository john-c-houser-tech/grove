//! Qt's painter, as far as the landscape needs it: images to draw into,
//! paths, gradients, and the painter's own calls, each declared here and
//! called as Qt wrote it. Below the declarations are the few safe types the
//! layers draw with.

#[cxx_qt::bridge]
pub mod ffi {
    // The methods are named q_... so their generated symbols differ from
    // those of cxx-qt-lib's own bindings of the same Qt classes.
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qimage.h");
        #[namespace = "rust::cxxqtlib1"]
        type QImageFormat = cxx_qt_lib::QImageFormat;
        include!("cxx-qt-lib/qpainter.h");
        #[namespace = "rust::cxxqtlib1"]
        type QPainterRenderHint = cxx_qt_lib::QPainterRenderHint;
        include!("cxx-qt-lib/qpen.h");
        type QPen = cxx_qt_lib::QPen;
        include!("cxx-qt-lib/qpointf.h");
        type QPointF = cxx_qt_lib::QPointF;
        include!("cxx-qt-lib/qrectf.h");
        type QRectF = cxx_qt_lib::QRectF;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qt.h");
        #[namespace = "Qt"]
        type ClipOperation = cxx_qt_lib::ClipOperation;
        #[namespace = "Qt"]
        type FillRule = cxx_qt_lib::FillRule;
        #[namespace = "Qt"]
        type PenStyle = cxx_qt_lib::PenStyle;
        #[namespace = "Qt"]
        type PenCapStyle = cxx_qt_lib::PenCapStyle;
        #[namespace = "Qt"]
        type PenJoinStyle = cxx_qt_lib::PenJoinStyle;

        include!(<QtGui/QImage>);
        /// A picture in memory.
        #[cxx_name = "QImage"]
        type Picture;
        /// An image of that many pixels, its contents not yet set.
        #[namespace = "rust::cxxqtlib1"]
        #[cxx_name = "make_unique"]
        fn new_image(width: i32, height: i32, format: QImageFormat) -> UniquePtr<Picture>;
        /// Sets every pixel to an ARGB value.
        #[cxx_name = "fill"]
        fn q_fill(self: Pin<&mut Picture>, pixel: u32);
        #[cxx_name = "width"]
        fn q_width(self: &Picture) -> i32;
        #[cxx_name = "height"]
        fn q_height(self: &Picture) -> i32;
        /// The bytes of one row of pixels, four to a pixel.
        ///
        /// # Safety
        /// `row` must be a row of the image; the bytes are the image's.
        #[cxx_name = "constScanLine"]
        unsafe fn q_const_scan_line(self: &Picture, row: i32) -> *const u8;
        /// One pixel as ARGB, not premultiplied.
        #[cxx_name = "pixel"]
        fn q_pixel(self: &Picture, x: i32, y: i32) -> u32;
        /// How many of the image's pixels make one of the display's units.
        #[cxx_name = "devicePixelRatio"]
        fn q_device_pixel_ratio(self: &Picture) -> f64;
        #[cxx_name = "setDevicePixelRatio"]
        fn q_set_device_pixel_ratio(self: Pin<&mut Picture>, ratio: f64);
        /// Writes the image to a file, in the format its name ends with.
        ///
        /// # Safety
        /// `format` must be null or a NUL-terminated format name.
        #[cxx_name = "save"]
        unsafe fn q_save(self: &Picture, file: &QString, format: *const c_char, quality: i32) -> bool;

        include!(<QtGui/QPainterPath>);
        /// An outline, to fill or to stroke.
        #[cxx_name = "QPainterPath"]
        type Outline;
        #[namespace = "rust::cxxqtlib1"]
        #[cxx_name = "make_unique"]
        fn new_path() -> UniquePtr<Outline>;
        #[cxx_name = "setFillRule"]
        fn q_set_fill_rule(self: Pin<&mut Outline>, rule: FillRule);
        #[cxx_name = "moveTo"]
        fn q_move_to(self: Pin<&mut Outline>, x: f64, y: f64);
        #[cxx_name = "lineTo"]
        fn q_line_to(self: Pin<&mut Outline>, x: f64, y: f64);
        #[cxx_name = "quadTo"]
        fn q_quad_to(self: Pin<&mut Outline>, cx: f64, cy: f64, x: f64, y: f64);
        #[cxx_name = "cubicTo"]
        fn q_cubic_to(self: Pin<&mut Outline>, c1x: f64, c1y: f64, c2x: f64, c2y: f64, x: f64, y: f64);
        #[cxx_name = "addEllipse"]
        fn q_add_ellipse(self: Pin<&mut Outline>, rectangle: &QRectF);
        #[cxx_name = "closeSubpath"]
        fn q_close_subpath(self: Pin<&mut Outline>);

        include!(<QtGui/QBrush>);
        /// Colours along a line, for filling with.
        #[cxx_name = "QLinearGradient"]
        type Blend;
        /// A gradient running from one point to another.
        #[namespace = "rust::cxxqtlib1"]
        #[cxx_name = "make_unique"]
        fn new_gradient(x1: f64, y1: f64, x2: f64, y2: f64) -> UniquePtr<Blend>;
        /// Puts a colour at a place along the gradient, from 0 to 1.
        #[cxx_name = "setColorAt"]
        fn q_set_color_at(self: Pin<&mut Blend>, position: f64, color: &QColor);

        /// What a shape is filled with.
        #[cxx_name = "QBrush"]
        type Filling;
        #[namespace = "rust::cxxqtlib1"]
        #[cxx_name = "make_unique"]
        fn new_brush(color: &QColor) -> UniquePtr<Filling>;
        #[namespace = "rust::cxxqtlib1"]
        #[cxx_name = "make_unique"]
        fn new_gradient_brush(gradient: &Blend) -> UniquePtr<Filling>;

        /// A pen of any width, which cxx-qt-lib's own cannot be given.
        #[namespace = "rust::cxxqtlib1"]
        #[cxx_name = "construct"]
        fn new_pen(brush: &Filling, width: f64, style: PenStyle, cap: PenCapStyle, join: PenJoinStyle) -> QPen;

        include!(<QtGui/QPainter>);
        /// Qt's painter, drawing into an image.
        #[cxx_name = "QPainter"]
        type Brushwork;
        /// What a painter can draw on, which an image is.
        type QPaintDevice;
        /// Starts painting on a device, which must outlive the painting.
        ///
        /// # Safety
        /// `device` must be valid until the painter is dropped, and not used
        /// meanwhile.
        #[cxx_name = "begin"]
        unsafe fn q_begin(self: Pin<&mut Brushwork>, device: *mut QPaintDevice) -> bool;
        #[cxx_name = "setRenderHint"]
        fn q_set_render_hint(self: Pin<&mut Brushwork>, hint: QPainterRenderHint, on: bool);
        #[cxx_name = "scale"]
        fn q_scale(self: Pin<&mut Brushwork>, x: f64, y: f64);
        #[cxx_name = "translate"]
        fn q_translate(self: Pin<&mut Brushwork>, x: f64, y: f64);
        #[cxx_name = "save"]
        fn q_save(self: Pin<&mut Brushwork>);
        #[cxx_name = "restore"]
        fn q_restore(self: Pin<&mut Brushwork>);
        #[cxx_name = "fillPath"]
        fn q_fill_path(self: Pin<&mut Brushwork>, path: &Outline, brush: &Filling);
        #[cxx_name = "fillRect"]
        fn q_fill_rect(self: Pin<&mut Brushwork>, rectangle: &QRectF, brush: &Filling);
        #[cxx_name = "strokePath"]
        fn q_stroke_path(self: Pin<&mut Brushwork>, path: &Outline, pen: &QPen);
        #[cxx_name = "setClipPath"]
        fn q_set_clip_path(self: Pin<&mut Brushwork>, path: &Outline, operation: ClipOperation);
        #[cxx_name = "setClipRect"]
        fn q_set_clip_rect(self: Pin<&mut Brushwork>, rectangle: &QRectF, operation: ClipOperation);
        #[cxx_name = "drawImage"]
        fn q_draw_image(self: Pin<&mut Brushwork>, at: &QPointF, image: &Picture);
    }
}

use std::marker::PhantomData;
use std::path::Path as FilePath;
use std::pin::Pin;

use cxx::UniquePtr;
use cxx_qt_lib::{
    ClipOperation, FillRule, PenCapStyle, PenJoinStyle, PenStyle, QColor, QImageFormat, QPainterRenderHint, QPointF,
    QRectF, QString,
};

/// A picture in memory, with see-through pixels where nothing is drawn.
pub struct Image(UniquePtr<ffi::Picture>);

// SAFETY: a QImage owns its pixels and may be handed from one thread to
// another; it is only ever used by the one that has it.
unsafe impl Send for Image {}

impl Image {
    /// A wholly see-through image of that many pixels.
    pub fn new(width: i32, height: i32) -> Self {
        let mut image = ffi::new_image(width.max(1), height.max(1), QImageFormat::Format_ARGB32_Premultiplied);
        image.pin_mut().q_fill(0);
        Self(image)
    }

    pub fn width(&self) -> i32 {
        self.0.q_width()
    }

    pub fn height(&self) -> i32 {
        self.0.q_height()
    }

    /// The bytes of one row of pixels, four to a pixel.
    pub fn row(&self, y: i32) -> &[u8] {
        assert!(
            (0..self.height()).contains(&y),
            "no row {y} in an image {} high",
            self.height()
        );
        // SAFETY: `y` is a row of the image, whose rows are four bytes a
        // pixel, and the bytes live as long as the image is borrowed.
        unsafe { std::slice::from_raw_parts(self.0.q_const_scan_line(y), self.width() as usize * 4) }
    }

    /// One pixel as (alpha, red, green, blue), not premultiplied.
    pub fn pixel(&self, x: i32, y: i32) -> (u8, u8, u8, u8) {
        let [alpha, red, green, blue] = self.0.q_pixel(x, y).to_be_bytes();
        (alpha, red, green, blue)
    }

    /// How many of the image's pixels make one of the display's units.
    pub fn device_pixel_ratio(&self) -> f64 {
        self.0.q_device_pixel_ratio()
    }

    pub fn set_device_pixel_ratio(&mut self, ratio: f64) {
        self.0.pin_mut().q_set_device_pixel_ratio(ratio);
    }

    /// Writes the image to a file, in the format its name ends with.
    /// Returns whether it was written.
    pub fn save(&self, file: &FilePath) -> bool {
        // SAFETY: a null format has Qt go by the file's name.
        unsafe {
            self.0
                .q_save(&QString::from(file.to_string_lossy().as_ref()), std::ptr::null(), -1)
        }
    }

    /// A painter drawing into this image, with smooth edges.
    pub fn painter(&mut self) -> Painter<'_> {
        let mut painter = Painter {
            painter: cxx_qt_lib::QPainter::new(),
            image: PhantomData,
        };
        // An image is a paint device, and that part of it comes first.
        let device = self.0.as_mut_ptr().cast::<ffi::QPaintDevice>();
        // SAFETY: the painter borrows the image for as long as it lives, so
        // the image outlives it and is not touched meanwhile.
        unsafe { painter.q().q_begin(device) };
        painter.q().q_set_render_hint(QPainterRenderHint::Antialiasing, true);
        painter
    }

    /// Qt's own image, for handing to Qt.
    pub fn qimage(&self) -> &ffi::Picture {
        &self.0
    }
}

/// An outline, to fill or to stroke. Overlapping parts of one outline count
/// as inside it, as the layers' shapes all want.
pub struct Path(UniquePtr<ffi::Outline>);

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl Path {
    pub fn new() -> Self {
        let mut path = ffi::new_path();
        path.pin_mut().q_set_fill_rule(FillRule::WindingFill);
        Self(path)
    }

    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.0.pin_mut().q_move_to(x, y);
        self
    }

    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.0.pin_mut().q_line_to(x, y);
        self
    }

    /// A curve to (x, y), bent toward (cx, cy).
    pub fn quad_to(&mut self, cx: f64, cy: f64, x: f64, y: f64) -> &mut Self {
        self.0.pin_mut().q_quad_to(cx, cy, x, y);
        self
    }

    /// A curve to (x, y), leaving toward the first point and arriving from
    /// the second.
    pub fn cubic_to(&mut self, c1: (f64, f64), c2: (f64, f64), x: f64, y: f64) -> &mut Self {
        self.0.pin_mut().q_cubic_to(c1.0, c1.1, c2.0, c2.1, x, y);
        self
    }

    /// An ellipse filling the rectangle at (x, y) of that size.
    pub fn add_ellipse(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        self.0.pin_mut().q_add_ellipse(&QRectF::new(x, y, width, height));
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.0.pin_mut().q_close_subpath();
        self
    }
}

/// Colours along a line, for filling with.
pub struct Gradient(UniquePtr<ffi::Blend>);

impl Gradient {
    /// A gradient running from (x1, y1) to (x2, y2).
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self(ffi::new_gradient(x1, y1, x2, y2))
    }

    /// Puts a colour at a place along the gradient, from 0 to 1.
    pub fn stop(mut self, position: f64, color: &QColor) -> Self {
        self.0.pin_mut().q_set_color_at(position, color);
        self
    }
}

/// What a shape is filled with: one colour, or a gradient.
pub struct Fill(UniquePtr<ffi::Filling>);

impl From<&QColor> for Fill {
    fn from(color: &QColor) -> Self {
        Self(ffi::new_brush(color))
    }
}

impl From<&Gradient> for Fill {
    fn from(gradient: &Gradient) -> Self {
        Self(ffi::new_gradient_brush(&gradient.0))
    }
}

/// Qt's painter, drawing into an image it borrows.
pub struct Painter<'image> {
    /// cxx-qt-lib's painter owns the object; the calls it lacks are made
    /// through this file's declarations of the same class.
    painter: UniquePtr<cxx_qt_lib::QPainter>,
    image: PhantomData<&'image mut Image>,
}

impl Painter<'_> {
    /// The painter, as this file declares it.
    fn q(&mut self) -> Pin<&mut ffi::Brushwork> {
        let painter: *mut cxx_qt_lib::QPainter = self.painter.as_mut_ptr();
        // SAFETY: both types stand for the one QPainter this owns, which is
        // never moved.
        unsafe { Pin::new_unchecked(&mut *painter.cast::<ffi::Brushwork>()) }
    }

    /// Draws everything after this larger or smaller.
    pub fn scale(&mut self, x: f64, y: f64) {
        self.q().q_scale(x, y);
    }

    /// Draws everything after this moved over.
    pub fn translate(&mut self, x: f64, y: f64) {
        self.q().q_translate(x, y);
    }

    /// Remembers the clip and the transform, until `restore`.
    pub fn save(&mut self) {
        self.q().q_save();
    }

    pub fn restore(&mut self) {
        self.q().q_restore();
    }

    pub fn fill_path(&mut self, path: &Path, fill: impl Into<Fill>) {
        self.q().q_fill_path(&path.0, &fill.into().0);
    }

    pub fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64, fill: impl Into<Fill>) {
        self.q().q_fill_rect(&QRectF::new(x, y, width, height), &fill.into().0);
    }

    /// Draws along a path with a pen as a canvas strokes: flat ends and
    /// mitred corners.
    pub fn stroke_path(&mut self, path: &Path, color: &QColor, width: f64) {
        let brush = ffi::new_brush(color);
        let mut pen = ffi::new_pen(
            &brush,
            width,
            PenStyle::SolidLine,
            PenCapStyle::FlatCap,
            PenJoinStyle::MiterJoin,
        );
        pen.set_miter_limit(10.0);
        self.q().q_stroke_path(&path.0, &pen);
    }

    /// Keeps what is drawn after this to the inside of a path, within any
    /// clip already set.
    pub fn clip_to_path(&mut self, path: &Path) {
        self.q().q_set_clip_path(&path.0, ClipOperation::IntersectClip);
    }

    /// Keeps what is drawn after this to a rectangle, within any clip
    /// already set.
    pub fn clip_to_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.q()
            .q_set_clip_rect(&QRectF::new(x, y, width, height), ClipOperation::IntersectClip);
    }

    /// Draws another image with its top left corner at (x, y).
    pub fn draw_image(&mut self, x: f64, y: f64, image: &Image) {
        self.q().q_draw_image(&QPointF::new(x, y), &image.0);
    }
}

/// Colours mixed and darkened the way QML does it, so a layer drawn here
/// agrees with what the window draws beside it.
pub mod color {
    use cxx_qt_lib::QColor;

    /// A colour from components, clamped as Qt.rgba() clamps them.
    pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> QColor {
        let unit = |v: f64| v.clamp(0.0, 1.0) as f32;
        QColor::from_rgba_f(unit(r), unit(g), unit(b), unit(a))
    }

    /// From `a` toward `b` by `t`, opaque.
    pub fn mix(a: &QColor, b: &QColor, t: f64) -> QColor {
        let towards = |from: f32, to: f32| f64::from(from) + (f64::from(to) - f64::from(from)) * t;
        rgba(
            towards(a.red_f(), b.red_f()),
            towards(a.green_f(), b.green_f()),
            towards(a.blue_f(), b.blue_f()),
            1.0,
        )
    }

    /// The same colour with another alpha.
    pub fn with_alpha(color: &QColor, alpha: f64) -> QColor {
        rgba(
            f64::from(color.red_f()),
            f64::from(color.green_f()),
            f64::from(color.blue_f()),
            alpha,
        )
    }

    /// Qt.darker(): 1.5 is half again as dark.
    pub fn darker(color: &QColor, factor: f64) -> QColor {
        color.darker((factor * 100.0).round() as i32)
    }

    /// Qt.lighter(): 1.5 is half again as light.
    pub fn lighter(color: &QColor, factor: f64) -> QColor {
        color.lighter((factor * 100.0).round() as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gradient_filled_shape_is_painted_into_an_image() {
        let mut image = Image::new(40, 40);
        {
            let mut painter = image.painter();
            let gradient = Gradient::new(0.0, 0.0, 0.0, 40.0)
                .stop(0.0, &QColor::from_rgb(255, 0, 0))
                .stop(1.0, &QColor::from_rgb(0, 0, 255));
            let mut triangle = Path::new();
            triangle
                .move_to(20.0, 0.0)
                .line_to(40.0, 40.0)
                .line_to(0.0, 40.0)
                .close();
            painter.fill_path(&triangle, &gradient);
        }
        let (alpha, red, _, blue) = image.pixel(20, 6);
        assert!(
            alpha == 255 && red > 200 && blue < 60,
            "near the top: {alpha} {red} {blue}"
        );
        let (alpha, red, _, blue) = image.pixel(20, 36);
        assert!(
            alpha == 255 && red < 60 && blue > 200,
            "near the bottom: {alpha} {red} {blue}"
        );
        // Outside the triangle nothing was painted.
        assert_eq!(image.pixel(1, 1), (0, 0, 0, 0));
        assert!(image.row(0).iter().take(8).all(|byte| *byte == 0));
        assert_eq!((image.width(), image.height()), (40, 40));
    }

    #[test]
    fn strokes_clips_and_images_are_drawn() {
        let white = QColor::from_rgb(255, 255, 255);
        let mut small = Image::new(4, 4);
        small.painter().fill_rect(0.0, 0.0, 4.0, 4.0, &white);

        let mut image = Image::new(40, 40);
        {
            let mut painter = image.painter();
            let mut line = Path::new();
            line.move_to(0.0, 10.0).line_to(40.0, 10.0);
            painter.stroke_path(&line, &white, 4.0);
            painter.save();
            painter.clip_to_rect(0.0, 20.0, 10.0, 20.0);
            painter.fill_rect(0.0, 0.0, 40.0, 40.0, &white);
            painter.restore();
            painter.draw_image(30.0, 30.0, &small);
        }
        assert_eq!(image.pixel(20, 10), (255, 255, 255, 255));
        assert_eq!(image.pixel(20, 14).0, 0);
        // Filled only inside the clip, which restore then lifted.
        assert_eq!(image.pixel(5, 30).0, 255);
        assert_eq!(image.pixel(15, 30).0, 0);
        assert_eq!(image.pixel(32, 32).0, 255);
    }
}
