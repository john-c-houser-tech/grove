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

    extern "RustQt" {
        /// The ploughed field in the foreground; see `field.rs`.
        #[qobject]
        #[base = QQuickItem]
        #[qml_element]
        #[qproperty(f64, covered_from, cxx_name = "coveredFrom")]
        // The horizon, as a fraction of the height.
        #[qproperty(f64, horizon)]
        // Where the field's far edge meets the meadow, in pixels from the top
        // (by default, near the bottom).
        #[qproperty(f64, ground_top, cxx_name = "groundTop", READ = ground_top, WRITE = set_ground_top, NOTIFY = ground_top_changed)]
        // Where the rows meet on the horizon, and where the plant stands, as
        // fractions of the width.
        #[qproperty(f64, vanish_x, cxx_name = "vanishX")]
        #[qproperty(f64, plant_x, cxx_name = "plantX")]
        // "far" draws the ground and the rows beyond the plant, "near" the
        // rows in front of it, which are drawn over the plant.
        #[qproperty(QString, part)]
        #[qproperty(QColor, soil_color, cxx_name = "soilColor")]
        // The sky at the horizon, for the haze over the far side of the field.
        #[qproperty(QColor, haze_color, cxx_name = "hazeColor")]
        // Snow lying on the field, from 0 to 1, and frost on the crests in
        // winter.
        #[qproperty(f64, snow)]
        #[qproperty(bool, frost)]
        // Where the field ends, out at 40 m, and where the plant's foot is,
        // in pixels from the top. They follow only the size and the horizon,
        // so they have a signal of their own: groundTop is usually bound to
        // farEdgeY, and sharing a signal with it would make that binding loop.
        #[qproperty(f64, far_edge_y, cxx_name = "farEdgeY", READ = far_edge_y, NOTIFY = view_changed)]
        #[qproperty(f64, plant_base_y, cxx_name = "plantBaseY", READ = plant_base_y, NOTIFY = view_changed)]
        type Field = super::FieldRust;

        #[cxx_name = "groundTop"]
        fn ground_top(self: &Field) -> f64;
        #[cxx_name = "setGroundTop"]
        fn set_ground_top(self: Pin<&mut Field>, ground_top: f64);
        #[cxx_name = "farEdgeY"]
        fn far_edge_y(self: &Field) -> f64;
        #[cxx_name = "plantBaseY"]
        fn plant_base_y(self: &Field) -> f64;

        #[qsignal]
        #[cxx_name = "groundTopChanged"]
        fn ground_top_changed(self: Pin<&mut Field>);
        /// The far edge or the plant's foot moved.
        #[qsignal]
        #[cxx_name = "viewChanged"]
        fn view_changed(self: Pin<&mut Field>);

        #[cxx_override]
        #[cxx_name = "updatePaintNode"]
        unsafe fn update_paint_node(
            self: Pin<&mut Field>,
            old: *mut QSGNode,
            data: *mut UpdatePaintNodeData,
        ) -> *mut QSGNode;
        #[cxx_override]
        #[cxx_name = "geometryChange"]
        fn geometry_change(self: Pin<&mut Field>, new_geometry: &QRectF, old_geometry: &QRectF);
        #[cxx_override]
        #[cxx_name = "itemChange"]
        fn item_change(self: Pin<&mut Field>, change: ItemChange, value: &ItemChangeData);
    }

    extern "RustQt" {
        /// The valley floor with the railway and the crossing's road; see
        /// `trackbed.rs`.
        #[qobject]
        #[base = QQuickItem]
        #[qml_element]
        #[qproperty(f64, covered_from, cxx_name = "coveredFrom")]
        // Where the horizon sits, as a fraction of the height.
        #[qproperty(f64, horizon)]
        // Where the crossing stands, as a fraction of the width.
        #[qproperty(f64, crossing_x, cxx_name = "crossingX")]
        #[qproperty(QColor, floor_color, cxx_name = "floorColor")]
        #[qproperty(QColor, ballast_color, cxx_name = "ballastColor")]
        #[qproperty(QColor, rail_color, cxx_name = "railColor")]
        #[qproperty(QColor, road_color, cxx_name = "roadColor")]
        // Beyond the crossing: the bare earth of the cut slope above the
        // road, the tipped earth of the fill slope below it, and the hillside
        // itself.
        #[qproperty(QColor, cut_color, cxx_name = "cutColor")]
        #[qproperty(QColor, fill_color, cxx_name = "fillColor")]
        #[qproperty(QColor, hill_color, cxx_name = "hillColor")]
        #[qproperty(QColor, haze_color, cxx_name = "hazeColor")]
        type TrackBed = super::TrackBedRust;

        #[cxx_override]
        #[cxx_name = "updatePaintNode"]
        unsafe fn update_paint_node(
            self: Pin<&mut TrackBed>,
            old: *mut QSGNode,
            data: *mut UpdatePaintNodeData,
        ) -> *mut QSGNode;
        #[cxx_override]
        #[cxx_name = "geometryChange"]
        fn geometry_change(self: Pin<&mut TrackBed>, new_geometry: &QRectF, old_geometry: &QRectF);
        #[cxx_override]
        #[cxx_name = "itemChange"]
        fn item_change(self: Pin<&mut TrackBed>, change: ItemChange, value: &ItemChangeData);
    }

    // Qt's own handling of the same events, which each override calls first.
    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "geometryChange"]
        fn base_geometry_change(self: Pin<&mut Hills>, new_geometry: &QRectF, old_geometry: &QRectF);
        #[inherit]
        #[cxx_name = "itemChange"]
        fn base_item_change(self: Pin<&mut Hills>, change: ItemChange, value: &ItemChangeData);
        #[inherit]
        #[cxx_name = "geometryChange"]
        fn base_geometry_change(self: Pin<&mut Field>, new_geometry: &QRectF, old_geometry: &QRectF);
        #[inherit]
        #[cxx_name = "itemChange"]
        fn base_item_change(self: Pin<&mut Field>, change: ItemChange, value: &ItemChangeData);
        #[inherit]
        #[cxx_name = "geometryChange"]
        fn base_geometry_change(self: Pin<&mut TrackBed>, new_geometry: &QRectF, old_geometry: &QRectF);
        #[inherit]
        #[cxx_name = "itemChange"]
        fn base_item_change(self: Pin<&mut TrackBed>, change: ItemChange, value: &ItemChangeData);
    }

    impl cxx_qt::Threading for Hills {}
    impl cxx_qt::Initialize for Hills {}
    impl cxx_qt::Threading for Field {}
    impl cxx_qt::Initialize for Field {}
    impl cxx_qt::Threading for TrackBed {}
    impl cxx_qt::Initialize for TrackBed {}
}

use core::pin::Pin;

use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QRectF, QString};

use super::layer::{self, Layer, Painting, State};
use super::{field, hills, trackbed};

/// What every layer's `Layer` has in common: where its item and its state
/// are.
macro_rules! layer_basics {
    () => {
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
    };
}

/// The events every layer answers the same way: Qt's own handling first,
/// then the shared layer's. `$resized` is what else the layer does when its
/// size has changed.
macro_rules! layer_events {
    ($name:ident, $resized:expr) => {
        impl qobject::$name {
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
                if layer::resized(new_geometry, old_geometry) {
                    let resized: fn(Pin<&mut Self>) = $resized;
                    resized(self.as_mut());
                    layer::schedule_render(self);
                }
            }

            fn item_change(mut self: Pin<&mut Self>, change: qobject::ItemChange, value: &qobject::ItemChangeData) {
                self.as_mut().base_item_change(change, value);
                layer::item_changed(self, change);
            }
        }
    };
}

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
    layer_basics!();

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

layer_events!(Hills, |_layer| {});

/// What the `Field` item keeps: its properties, and its picture's state.
pub struct FieldRust {
    covered_from: f64,
    horizon: f64,
    /// Negative until it is set, which stands for near the bottom.
    ground_top: f64,
    vanish_x: f64,
    plant_x: f64,
    part: QString,
    soil_color: QColor,
    haze_color: QColor,
    snow: f64,
    frost: bool,
    state: State,
}

impl Default for FieldRust {
    fn default() -> Self {
        let color = |hex: &str| QColor::try_from(hex).unwrap_or_default();
        Self {
            covered_from: -1.0,
            horizon: 0.6,
            ground_top: -1.0,
            vanish_x: 0.5,
            plant_x: 0.33,
            part: QString::from("far"),
            soil_color: color("#2a2018"),
            haze_color: color("#9fb7c4"),
            snow: 0.0,
            frost: false,
            state: State::default(),
        }
    }
}

impl Layer for qobject::Field {
    layer_basics!();

    fn painting(&self) -> Painting {
        let rust = self.rust();
        let look = field::Look {
            horizon: rust.horizon,
            ground_top: self.ground_top(),
            vanish_x: rust.vanish_x,
            plant_x: rust.plant_x,
            part: field::Part::named(&rust.part.to_string()),
            soil_color: rust.soil_color.clone(),
            haze_color: rust.haze_color.clone(),
            snow: rust.snow,
            frost: rust.frost,
        };
        Box::new(move |painter, width, height| field::paint(painter, &look, width, height))
    }
}

impl cxx_qt::Initialize for qobject::Field {
    fn initialize(mut self: Pin<&mut Self>) {
        layer::initialize(self.as_mut());
        // Any property that changes has the layer drawn again.
        self.as_mut().on_covered_from_changed(layer::schedule_render).release();
        self.as_mut().on_ground_top_changed(layer::schedule_render).release();
        self.as_mut().on_vanish_x_changed(layer::schedule_render).release();
        self.as_mut().on_plant_x_changed(layer::schedule_render).release();
        self.as_mut().on_part_changed(layer::schedule_render).release();
        self.as_mut().on_soil_color_changed(layer::schedule_render).release();
        self.as_mut().on_haze_color_changed(layer::schedule_render).release();
        self.as_mut().on_snow_changed(layer::schedule_render).release();
        self.as_mut().on_frost_changed(layer::schedule_render).release();
        // The far edge and the plant's foot move with the horizon too.
        self.as_mut()
            .on_horizon_changed(|mut field| {
                field.as_mut().view_changed();
                layer::schedule_render(field);
            })
            .release();
    }
}

impl qobject::Field {
    fn ground_top(&self) -> f64 {
        let ground_top = self.rust().ground_top;
        if ground_top >= 0.0 {
            ground_top
        } else {
            self.item().height() * 0.874
        }
    }

    fn set_ground_top(mut self: Pin<&mut Self>, ground_top: f64) {
        if ground_top == self.rust().ground_top {
            return;
        }
        self.as_mut().rust_mut().ground_top = ground_top;
        self.ground_top_changed();
    }

    fn far_edge_y(&self) -> f64 {
        field::far_edge_y(self.item().height(), self.rust().horizon)
    }

    fn plant_base_y(&self) -> f64 {
        field::plant_base_y(self.item().height(), self.rust().horizon)
    }
}

// The far edge and the plant's foot move with the size.
layer_events!(Field, |field| field.view_changed());

/// What the `TrackBed` item keeps: its properties, and its picture's state.
pub struct TrackBedRust {
    covered_from: f64,
    horizon: f64,
    crossing_x: f64,
    floor_color: QColor,
    ballast_color: QColor,
    rail_color: QColor,
    road_color: QColor,
    cut_color: QColor,
    fill_color: QColor,
    hill_color: QColor,
    haze_color: QColor,
    state: State,
}

impl Default for TrackBedRust {
    fn default() -> Self {
        let color = |hex: &str| QColor::try_from(hex).unwrap_or_default();
        Self {
            covered_from: -1.0,
            horizon: 0.88,
            crossing_x: 0.68,
            floor_color: color("#33503f"),
            ballast_color: color("#26382d"),
            rail_color: color("#6d7f78"),
            road_color: color("#7a7258"),
            cut_color: color("#4a4232"),
            fill_color: color("#3d3a2c"),
            hill_color: color("#44605a"),
            haze_color: color("#9fb7c4"),
            state: State::default(),
        }
    }
}

impl Layer for qobject::TrackBed {
    layer_basics!();

    fn painting(&self) -> Painting {
        let rust = self.rust();
        let look = trackbed::Look {
            horizon: rust.horizon,
            crossing_x: rust.crossing_x,
            floor_color: rust.floor_color.clone(),
            ballast_color: rust.ballast_color.clone(),
            rail_color: rust.rail_color.clone(),
            road_color: rust.road_color.clone(),
            cut_color: rust.cut_color.clone(),
            fill_color: rust.fill_color.clone(),
            hill_color: rust.hill_color.clone(),
            haze_color: rust.haze_color.clone(),
        };
        Box::new(move |painter, width, height| trackbed::paint(painter, &look, width, height))
    }
}

impl cxx_qt::Initialize for qobject::TrackBed {
    fn initialize(mut self: Pin<&mut Self>) {
        layer::initialize(self.as_mut());
        // Any property that changes has the layer drawn again.
        self.as_mut().on_covered_from_changed(layer::schedule_render).release();
        self.as_mut().on_horizon_changed(layer::schedule_render).release();
        self.as_mut().on_crossing_x_changed(layer::schedule_render).release();
        self.as_mut().on_floor_color_changed(layer::schedule_render).release();
        self.as_mut().on_ballast_color_changed(layer::schedule_render).release();
        self.as_mut().on_rail_color_changed(layer::schedule_render).release();
        self.as_mut().on_road_color_changed(layer::schedule_render).release();
        self.as_mut().on_cut_color_changed(layer::schedule_render).release();
        self.as_mut().on_fill_color_changed(layer::schedule_render).release();
        self.as_mut().on_hill_color_changed(layer::schedule_render).release();
        self.as_mut().on_haze_color_changed(layer::schedule_render).release();
    }
}

layer_events!(TrackBed, |_layer| {});
