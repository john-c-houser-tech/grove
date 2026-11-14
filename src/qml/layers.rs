//! The landscape layers drawn by Rust, as QML items: each is a still picture
//! of part of the land, drawn with Qt's painter on another thread and shown
//! as a single texture. `layer.rs` has what they share; each layer's drawing
//! is in a file of its own.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++Qt" {
        include!(<QtQuick/QQuickItem>);
        #[qobject]
        type QQuickItem;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qrectf.h");
        type QRectF = cxx_qt_lib::QRectF;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!(<QtGui/QImage>);
        #[cxx_name = "QImage"]
        type Picture = crate::qml::paint::ffi::Picture;

        // Types Qt declares inside its classes, described in `layer.rs`.
        #[namespace = "QQuickItem"]
        type Flag = crate::qml::layer::Flag;
        #[namespace = "QQuickItem"]
        type ItemChange = crate::qml::layer::ItemChange;
        #[namespace = "QQuickItem"]
        type ItemChangeData = crate::qml::layer::ItemChangeData;
        #[namespace = "QQuickItem"]
        type UpdatePaintNodeData = crate::qml::layer::UpdatePaintNodeData;

        #[cxx_name = "setFlag"]
        fn set_flag(self: Pin<&mut QQuickItem>, flag: Flag, enabled: bool);
        /// The window the item is in; null when it is in none.
        fn window(self: &QQuickItem) -> *mut QQuickWindow;
        fn width(self: &QQuickItem) -> f64;
        fn height(self: &QQuickItem) -> f64;
        /// Asks for `updatePaintNode` to be called before the next frame.
        fn update(self: Pin<&mut QQuickItem>);

        include!(<QtQuick/QQuickWindow>);
        type QQuickWindow;
        /// How many pixels of the display make one of the window's units.
        #[cxx_name = "effectiveDevicePixelRatio"]
        fn effective_device_pixel_ratio(self: &QQuickWindow) -> f64;
        /// A texture holding a copy of an image, which the caller owns.
        #[cxx_name = "createTextureFromImage"]
        fn create_texture_from_image(self: &QQuickWindow, image: &Picture) -> *mut QSGTexture;

        include!(<QtQuick/QSGSimpleTextureNode>);
        /// A node of the scene graph.
        type QSGNode;
        type QSGTexture;
        #[namespace = "QSGTexture"]
        type Filtering = crate::qml::layer::Filtering;
        /// A node that shows one texture in a rectangle.
        type QSGSimpleTextureNode;
        #[namespace = "rust::cxxqtlib1"]
        #[cxx_name = "make_unique"]
        fn new_texture_node() -> UniquePtr<QSGSimpleTextureNode>;
        /// Has the node delete its texture when it gets another or goes.
        #[cxx_name = "setOwnsTexture"]
        fn set_owns_texture(self: Pin<&mut QSGSimpleTextureNode>, owns: bool);
        #[cxx_name = "setFiltering"]
        fn set_filtering(self: Pin<&mut QSGSimpleTextureNode>, filtering: Filtering);
        /// Shows a texture, which the node comes to own if it owns textures.
        ///
        /// # Safety
        /// `texture` must be a valid texture no one else owns.
        #[cxx_name = "setTexture"]
        unsafe fn set_texture(self: Pin<&mut QSGSimpleTextureNode>, texture: *mut QSGTexture);
        /// Where the texture is shown, in the item's units.
        #[cxx_name = "setRect"]
        fn set_rect(self: Pin<&mut QSGSimpleTextureNode>, rectangle: &QRectF);
    }

    extern "RustQt" {
        /// The view beyond the garden: distant granite peaks, rolling ridges
        /// and a nearer hillside; see `hills.rs`.
        #[qobject]
        #[base = QQuickItem]
        #[qml_element]
        // The height in pixels from which this layer is hidden by what
        // stands in front of it; negative when nothing does.
        #[qproperty(f64, covered_from, cxx_name = "coveredFrom")]
        // Where the horizon sits, as a fraction of the height.
        #[qproperty(f64, horizon)]
        // "back" draws the peaks and ridges, "front" the hillside nearest to
        // us, so something can pass between them; "all" draws both.
        #[qproperty(QString, part)]
        // How low the snow reaches on the far peaks, as a fraction of their
        // height.
        #[qproperty(f64, snowline)]
        #[qproperty(QColor, far_color, cxx_name = "farColor")]
        #[qproperty(QColor, mid_color, cxx_name = "midColor")]
        #[qproperty(QColor, near_color, cxx_name = "nearColor")]
        #[qproperty(QColor, haze_color, cxx_name = "hazeColor")]
        // The far range is bare granite: lit rock, rock in shadow, and forest
        // along its foot. Which side of each peak is in shadow follows the
        // sun (1 when the right-hand faces are shaded), and relief is how
        // strong the shadows are, from 0 under cloud or at night to about 0.7
        // in clear sun.
        #[qproperty(QColor, shade_color, cxx_name = "shadeColor")]
        #[qproperty(QColor, forest_color, cxx_name = "forestColor")]
        #[qproperty(i32, shadow_side, cxx_name = "shadowSide")]
        #[qproperty(f64, relief)]
        // How much snow the range carries, from a few high patches (0) to
        // fully white (1).
        #[qproperty(f64, snow_cover, cxx_name = "snowCover")]
        #[qproperty(QColor, snow_color, cxx_name = "snowColor")]
        type Hills = super::HillsRust;

        #[cxx_override]
        #[cxx_name = "updatePaintNode"]
        unsafe fn update_paint_node(
            self: Pin<&mut Hills>,
            old: *mut QSGNode,
            data: *mut UpdatePaintNodeData,
        ) -> *mut QSGNode;
        #[cxx_override]
        #[cxx_name = "geometryChange"]
        fn geometry_change(self: Pin<&mut Hills>, new_geometry: &QRectF, old_geometry: &QRectF);
        #[cxx_override]
        #[cxx_name = "itemChange"]
        fn item_change(self: Pin<&mut Hills>, change: ItemChange, value: &ItemChangeData);
    }

    // Qt's own handling of the same events, which each override calls first.
    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "geometryChange"]
        fn base_geometry_change(self: Pin<&mut Hills>, new_geometry: &QRectF, old_geometry: &QRectF);
        #[inherit]
        #[cxx_name = "itemChange"]
        fn base_item_change(self: Pin<&mut Hills>, change: ItemChange, value: &ItemChangeData);
    }

    impl cxx_qt::Threading for Hills {}
    impl cxx_qt::Initialize for Hills {}
}

use core::pin::Pin;

use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QRectF, QString};

use super::hills;
use super::layer::{self, Layer, Painting, State};

/// What the `Hills` item keeps: its properties, and its picture's state.
pub struct HillsRust {
    covered_from: f64,
    horizon: f64,
    part: QString,
    snowline: f64,
    far_color: QColor,
    mid_color: QColor,
    near_color: QColor,
    haze_color: QColor,
    shade_color: QColor,
    forest_color: QColor,
    shadow_side: i32,
    relief: f64,
    snow_cover: f64,
    snow_color: QColor,
    state: State,
}

impl Default for HillsRust {
    fn default() -> Self {
        let color = |hex: &str| QColor::try_from(hex).unwrap_or_default();
        Self {
            covered_from: -1.0,
            horizon: 0.88,
            part: QString::from("all"),
            snowline: 0.65,
            far_color: color("#6b7f9c"),
            mid_color: color("#46605a"),
            near_color: color("#30493b"),
            haze_color: color("#9fb7c4"),
            shade_color: color("#4d5a6c"),
            forest_color: color("#2f3e35"),
            shadow_side: 1,
            relief: 0.5,
            snow_cover: 0.3,
            snow_color: color("#f2f6fa"),
            state: State::default(),
        }
    }
}

impl Layer for qobject::Hills {
    fn item(&self) -> &qobject::QQuickItem {
        self.upcast()
    }

    fn item_mut(self: Pin<&mut Self>) -> Pin<&mut qobject::QQuickItem> {
        self.upcast_pin()
    }

    fn state(self: Pin<&mut Self>) -> &mut State {
        // SAFETY: the state is plain data, moved by no one.
        unsafe { &mut self.rust_mut().get_unchecked_mut().state }
    }

    fn covered_from(&self) -> f64 {
        self.rust().covered_from
    }

    fn painting(&self) -> Painting {
        let rust = self.rust();
        let look = hills::Look {
            horizon: rust.horizon,
            part: hills::Part::named(&rust.part.to_string()),
            snowline: rust.snowline,
            far_color: rust.far_color.clone(),
            mid_color: rust.mid_color.clone(),
            near_color: rust.near_color.clone(),
            haze_color: rust.haze_color.clone(),
            shade_color: rust.shade_color.clone(),
            forest_color: rust.forest_color.clone(),
            shadow_side: rust.shadow_side,
            relief: rust.relief,
            snow_cover: rust.snow_cover,
            snow_color: rust.snow_color.clone(),
        };
        Box::new(move |painter, width, height| hills::paint(painter, &look, width, height))
    }
}

impl cxx_qt::Initialize for qobject::Hills {
    fn initialize(mut self: Pin<&mut Self>) {
        layer::initialize(self.as_mut());
        // Any property that changes has the layer drawn again.
        self.as_mut().on_covered_from_changed(layer::schedule_render).release();
        self.as_mut().on_horizon_changed(layer::schedule_render).release();
        self.as_mut().on_part_changed(layer::schedule_render).release();
        self.as_mut().on_snowline_changed(layer::schedule_render).release();
        self.as_mut().on_far_color_changed(layer::schedule_render).release();
        self.as_mut().on_mid_color_changed(layer::schedule_render).release();
        self.as_mut().on_near_color_changed(layer::schedule_render).release();
        self.as_mut().on_haze_color_changed(layer::schedule_render).release();
        self.as_mut().on_shade_color_changed(layer::schedule_render).release();
        self.as_mut().on_forest_color_changed(layer::schedule_render).release();
        self.as_mut().on_shadow_side_changed(layer::schedule_render).release();
        self.as_mut().on_relief_changed(layer::schedule_render).release();
        self.as_mut().on_snow_cover_changed(layer::schedule_render).release();
        self.as_mut().on_snow_color_changed(layer::schedule_render).release();
    }
}

impl qobject::Hills {
    /// # Safety
    /// `old` must be null or the node this item returned before.
    unsafe fn update_paint_node(
        self: Pin<&mut Self>,
        old: *mut qobject::QSGNode,
        _data: *mut qobject::UpdatePaintNodeData,
    ) -> *mut qobject::QSGNode {
        // SAFETY: passed on as received.
        unsafe { layer::update_paint_node(self, old) }
    }

    fn geometry_change(mut self: Pin<&mut Self>, new_geometry: &QRectF, old_geometry: &QRectF) {
        self.as_mut().base_geometry_change(new_geometry, old_geometry);
        layer::geometry_changed(self, new_geometry, old_geometry);
    }

    fn item_change(mut self: Pin<&mut Self>, change: qobject::ItemChange, value: &qobject::ItemChangeData) {
        self.as_mut().base_item_change(change, value);
        layer::item_changed(self, change);
    }
}
