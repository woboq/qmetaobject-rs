use qmetaobject::scenegraph::SGNode;
use qmetaobject::{QColor, QQuickItem, QRectF};

qrc! {
    init_ressource,
    "scenegraph/graph" {
//        "main.qml",
        "shaders/noisy.vsh",
        "shaders/noisy.fsh",
        "shaders/line.vsh",
        "shaders/line.fsh",
    }
}

// Ideally, everything should be possible to do in plain rust.
// However, there is quite some API to expose.

cpp! {{
#include "src/linenode.cpp"
#include "src/noisynode.cpp"
#include "src/gridnode.cpp"
#include <QtQuick/QQuickItem>
}}

pub enum NoisyNode {}

pub fn create_noisy_node(s: &mut SGNode<NoisyNode>, ctx: &dyn QQuickItem) {
    init_ressource();
    let item_ptr = ctx.get_cpp_object();
    cpp!(unsafe [s as "NoisyNode**", item_ptr as "QQuickItem*"] {
        if (!*s && item_ptr) {
            *s = new NoisyNode(item_ptr->window());
        }
    });
}

pub fn noisy_node_set_rect(s: &mut SGNode<NoisyNode>, rect: QRectF) {
    cpp!(unsafe [s as "NoisyNode**", rect as "QRectF"] {
        if (*s) {
            (*s)->setRect(rect);
        }
    });
}

pub enum GridNode {}
pub fn update_grid_node(s: &mut SGNode<GridNode>, rect: QRectF) {
    cpp!(unsafe [s as "GridNode**", rect as "QRectF"] {
        if (!*s) *s = new GridNode;
        (*s)->setRect(rect);
    });
}

pub enum LineNode {}
pub fn create_line_node(s: &mut SGNode<LineNode>, size: f32, spread: f32, color: QColor) {
    init_ressource();
    cpp!(unsafe [s as "LineNode**", size as "float", spread as "float", color as "QColor"] {
        if (!*s) *s = new LineNode(size, spread, color);
    });
}

pub fn update_line_node(s: &mut SGNode<LineNode>, rect: QRectF, samples: &[f64]) {
    let samples_ptr = samples.as_ptr();
    let samples_len = samples.len();
    cpp!(unsafe [s as "LineNode**", rect as "QRectF", samples_ptr as "double*",
                 samples_len as "std::size_t"] {
        if (!*s) return;
        QList<qreal> samples;
        samples.reserve(samples_len);
        std::copy(samples_ptr, samples_ptr + samples_len, std::back_inserter(samples));
        (*s)->updateGeometry(rect, samples);
    });
}
