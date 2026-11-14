//! What the landscape layers share. A layer is a still picture of part of
//! the land, drawn with Qt's painter on another thread and shown as a single
//! texture. Only the rows that hold something are kept, and nothing is drawn
//! below `coveredFrom`, where the ground in front hides it; once the picture
//! is on the GPU the copy here is let go.

use core::pin::Pin;

use cxx::UniquePtr;
use cxx_qt::Threading;
use cxx_qt_lib::QRectF;

use super::layers::qobject::{self, QQuickItem, QSGNode, QSGSimpleTextureNode};
use super::paint::{Image, Painter};

/// One of Qt's enumerations that is declared inside a class, which the
/// bridge cannot declare itself: a whole number on both sides.
macro_rules! nested_enum {
    ($(#[$doc:meta])* $name:ident, $id:literal, { $($(#[$value_doc:meta])* $value:ident = $number:literal),+ $(,)? }) => {
        $(#[$doc])*
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name(i32);

        impl $name {
            $($(#[$value_doc])* pub const $value: Self = Self($number);)+
        }

        // SAFETY: the C++ type of this name is an enumeration the size of an
        // int, which is passed by value as one.
        unsafe impl cxx::ExternType for $name {
            type Id = cxx::type_id!($id);
            type Kind = cxx::kind::Trivial;
        }
    };
}

nested_enum!(
    /// What an item is, as Qt's scene sees it.
    Flag, "QQuickItem::Flag", {
        /// The item has something to show, so `updatePaintNode` is called.
        HAS_CONTENTS = 0x08,
    }
);

nested_enum!(
    /// What changed about an item.
    ItemChange, "QQuickItem::ItemChange", {
        /// It came into a window, or left one.
        SCENE = 2,
        /// Its display's scale changed.
        DEVICE_PIXEL_RATIO = 9,
    }
);

nested_enum!(
    /// How a texture is sampled when it is not shown pixel for pixel.
    Filtering, "QSGTexture::Filtering", {
        LINEAR = 2,
    }
);

/// One of Qt's classes that is declared inside another, only ever handled
/// by reference.
macro_rules! nested_opaque {
    ($(#[$doc:meta])* $name:ident, $id:literal) => {
        $(#[$doc])*
        #[repr(C)]
        pub struct $name {
            _private: [u8; 0],
            _pinned: core::marker::PhantomPinned,
        }

        // SAFETY: the C++ type of this name is only ever behind a pointer or
        // a reference here.
        unsafe impl cxx::ExternType for $name {
            type Id = cxx::type_id!($id);
            type Kind = cxx::kind::Opaque;
        }
    };
}

nested_opaque!(
    /// The details of a change to an item, which the layers never read.
    ItemChangeData, "QQuickItem::ItemChangeData"
);
nested_opaque!(
    /// What Qt passes along with a paint node, which the layers never read.
    UpdatePaintNodeData, "QQuickItem::UpdatePaintNodeData"
);

/// Draws a layer at a size in the window's units. It holds everything it
/// needs by value, since it runs on another thread.
pub type Painting = Box<dyn FnOnce(&mut Painter<'_>, f64, f64) + Send>;

/// A drawn layer: its image, and where the image starts, in the window's
/// units from the top of the item.
pub struct Picture {
    pub image: Image,
    pub top: f64,
}

/// Where a layer's picture stands.
#[derive(Default)]
pub struct State {
    /// A drawing has been asked for and will start once the current event is
    /// handled, however many properties change meanwhile.
    scheduled: bool,
    /// A drawing is under way on another thread.
    drawing: bool,
    /// Something changed during that drawing, so another follows it.
    again: bool,
    /// A finished drawing waiting to become a texture; `Some(None)` when
    /// there was nothing to draw, and the texture should go.
    ready: Option<Option<Picture>>,
}

/// A QML item that is a layer.
pub trait Layer: Threading + Sized + 'static {
    /// The item, as Qt's own class.
    fn item(&self) -> &QQuickItem;
    fn item_mut(self: Pin<&mut Self>) -> Pin<&mut QQuickItem>;
    fn state(self: Pin<&mut Self>) -> &mut State;
    /// The height in pixels from which the layer is hidden; negative when
    /// it is not.
    fn covered_from(&self) -> f64;
    /// A drawing of the layer as it looks now.
    fn painting(&self) -> Painting;
}

/// Sets a new layer up as an item with something to show.
pub fn initialize<L: Layer>(layer: Pin<&mut L>) {
    layer.item_mut().set_flag(Flag::HAS_CONTENTS, true);
}

/// Draws the layer again once the current event is handled, however many
/// properties change meanwhile.
pub fn schedule_render<L: Layer>(mut layer: Pin<&mut L>) {
    if std::mem::replace(&mut layer.as_mut().state().scheduled, true) {
        return;
    }
    let _ = layer.qt_thread().queue(render);
}

/// Starts a drawing of the layer as it looks now.
fn render<L: Layer>(mut layer: Pin<&mut L>) {
    layer.as_mut().state().scheduled = false;
    // One drawing at a time; a change meanwhile is drawn when it is done.
    if layer.as_mut().state().drawing {
        layer.as_mut().state().again = true;
        return;
    }
    let (width, height) = (layer.item().width(), layer.item().height());
    // SAFETY: the item's window, when it has one, outlives this call.
    let Some(window) = (unsafe { layer.item().window().as_ref() }) else {
        return;
    };
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let ratio = window.effective_device_pixel_ratio();
    let (painting, covered_from) = (layer.painting(), layer.covered_from());
    layer.as_mut().state().drawing = true;
    // The thread gets everything by value and hands its picture back to the
    // GUI thread; an item destroyed meanwhile simply never sees it.
    let gui = layer.qt_thread();
    std::thread::spawn(move || {
        let picture = draw(painting, width, height, covered_from, ratio);
        let _ = gui.queue(move |layer| finished(layer, picture));
    });
}

/// Takes a finished drawing, to be shown with the next frame.
fn finished<L: Layer>(mut layer: Pin<&mut L>, picture: Option<Picture>) {
    let state = layer.as_mut().state();
    state.drawing = false;
    state.ready = Some(picture);
    let again = std::mem::take(&mut state.again);
    layer.as_mut().item_mut().update();
    if again {
        render(layer);
    }
}

/// Draws a painting as a layer shows it: at the display's scale, with the
/// empty rows above and below trimmed and nothing below `covered_from`.
/// `None` if nothing was drawn.
pub fn draw(painting: Painting, width: f64, height: f64, covered_from: f64, ratio: f64) -> Option<Picture> {
    let bottom = if covered_from >= 0.0 {
        height.min(covered_from)
    } else {
        height
    };
    let (pixels_wide, pixels_high) = ((width * ratio).ceil() as i32, (bottom * ratio).ceil() as i32);
    if pixels_wide <= 0 || pixels_high <= 0 {
        return None;
    }
    let mut image = Image::new(pixels_wide, pixels_high);
    {
        let mut painter = image.painter();
        painter.scale(ratio, ratio);
        painting(&mut painter, width, height);
    }

    // Keep only the rows with something in them.
    let empty = |y: i32| image.row(y).iter().all(|byte| *byte == 0);
    let (mut top, mut end) = (0, image.height());
    while top < end && empty(top) {
        top += 1;
    }
    while end > top && empty(end - 1) {
        end -= 1;
    }
    if top == end {
        return None;
    }
    let mut kept = Image::new(pixels_wide, end - top);
    kept.painter().draw_image(0.0, -f64::from(top), &image);
    kept.set_device_pixel_ratio(ratio);
    Some(Picture {
        image: kept,
        top: f64::from(top) / ratio,
    })
}

/// Shows a finished drawing as the item's texture: Qt calls this on its
/// render thread, with the GUI thread held, before a frame.
///
/// # Safety
/// `old` must be null or the node this layer returned last time.
pub unsafe fn update_paint_node<L: Layer>(mut layer: Pin<&mut L>, old: *mut QSGNode) -> *mut QSGNode {
    let Some(ready) = layer.as_mut().state().ready.take() else {
        return old;
    };
    // The only node a layer ever returns is a texture node, and a texture
    // node is a node first.
    let node = old.cast::<QSGSimpleTextureNode>();
    let Some(picture) = ready else {
        if !node.is_null() {
            // SAFETY: the scene graph hands the node back to be deleted.
            drop(unsafe { UniquePtr::from_raw(node) });
        }
        return std::ptr::null_mut();
    };
    // SAFETY: an item being drawn is in a window.
    let Some(window) = (unsafe { layer.item().window().as_ref() }) else {
        return old;
    };
    let node = if node.is_null() {
        let mut node = qobject::new_texture_node();
        node.pin_mut().set_owns_texture(true);
        node.pin_mut().set_filtering(Filtering::LINEAR);
        // The scene graph owns the node from here on.
        node.into_raw()
    } else {
        node
    };
    let ratio = picture.image.device_pixel_ratio();
    let (width, height) = (
        f64::from(picture.image.width()) / ratio,
        f64::from(picture.image.height()) / ratio,
    );
    // The texture takes its own copy, and this one goes when `picture` does.
    let texture = window.create_texture_from_image(picture.image.qimage());
    // SAFETY: `node` is a live texture node, never moved, and it owns the
    // texture just made for it.
    unsafe {
        let mut node = Pin::new_unchecked(&mut *node);
        node.as_mut().set_texture(texture);
        node.as_mut().set_rect(&QRectF::new(0.0, picture.top, width, height));
    }
    node.cast::<QSGNode>()
}

/// Whether a change of geometry changed the item's size, so that its
/// picture no longer fits.
pub fn resized(new_geometry: &QRectF, old_geometry: &QRectF) -> bool {
    new_geometry.width() != old_geometry.width() || new_geometry.height() != old_geometry.height()
}

/// The item came into a window, or moved to a display of another scale.
pub fn item_changed<L: Layer>(layer: Pin<&mut L>, change: ItemChange) {
    if change == ItemChange::SCENE || change == ItemChange::DEVICE_PIXEL_RATIO {
        schedule_render(layer);
    }
}

#[cfg(test)]
mod tests {
    use cxx_qt_lib::QColor;

    use super::*;

    fn band() -> Painting {
        Box::new(|painter, width, _| painter.fill_rect(0.0, 30.0, width, 20.0, &QColor::from_rgb(255, 0, 0)))
    }

    #[test]
    fn a_layer_keeps_only_the_rows_with_something_in_them() {
        let picture = draw(band(), 100.0, 100.0, -1.0, 1.0).unwrap();
        assert_eq!(picture.top, 30.0);
        assert_eq!((picture.image.width(), picture.image.height()), (100, 20));
        assert_eq!(picture.image.pixel(50, 10), (255, 255, 0, 0));
    }

    #[test]
    fn a_layer_is_drawn_at_the_displays_scale() {
        let picture = draw(band(), 100.0, 100.0, -1.0, 2.0).unwrap();
        assert_eq!(picture.top, 30.0);
        assert_eq!((picture.image.width(), picture.image.height()), (200, 40));
        assert_eq!(picture.image.device_pixel_ratio(), 2.0);
    }

    #[test]
    fn nothing_is_drawn_where_the_ground_covers_it() {
        let all: Painting =
            Box::new(|painter, width, height| painter.fill_rect(0.0, 0.0, width, height, &QColor::from_rgb(255, 0, 0)));
        let picture = draw(all, 100.0, 100.0, 60.0, 1.0).unwrap();
        assert_eq!((picture.image.width(), picture.image.height()), (100, 60));
        // Nothing drawn, nothing kept.
        assert!(draw(Box::new(|_, _, _| {}), 100.0, 100.0, -1.0, 1.0).is_none());
    }
}
