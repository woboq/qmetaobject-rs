#![allow(non_snake_case)]
#![allow(unused_variables)]

use cstr::cstr;

#[cfg(not(no_qt))]
use cpp::cpp;

#[cfg(no_qt)]
mod no_qt {
    pub fn panic<T>() -> T {
        panic!("This example is not supported on Qt 6 and above")
    }
}

#[cfg(no_qt)]
macro_rules! cpp {
    {{ $($t:tt)* }} => {};
    {$(unsafe)? [$($a:tt)*] -> $ret:ty as $b:tt { $($t:tt)* } } => {
        crate::no_qt::panic::<$ret>()
    };
    { $($t:tt)* } => {
        crate::no_qt::panic::<()>()
    };
}

use qmetaobject::prelude::*;
use qmetaobject::scenegraph::*;

mod nodes;

cpp! {{
    #include <QtQuick/QQuickItem>
}}

#[derive(Default, QObject)]
struct Graph {
    base: qt_base_class!(trait QQuickItem),

    m_samples: Vec<f64>,
    m_samplesChanged: bool,
    m_geometryChanged: bool,

    appendSample: qt_method!(fn(&mut self, value: f64)),
    removeFirstSample: qt_method!(
        fn removeFirstSample(&mut self) {
            self.m_samples.drain(0..1);
            self.m_samplesChanged = true;
            (self as &dyn QQuickItem).update();
        }
    ),
}

// Example of adding an enum wrapper.

/// Wrapper for [`QQuickItem::Flag`][enum] enum.
///
/// [enum]: https://doc.qt.io/qt-5/qquickitem.html#Flag-enum
#[allow(unused)]
#[repr(C)]
enum QQuickItemFlag {
    ItemClipsChildrenToShape = 0x01,
    ItemAcceptsInputMethod = 0x02,
    ItemIsFocusScope = 0x04,
    ItemHasContents = 0x08,
    ItemAcceptsDrops = 0x10,
}

impl Graph {
    // Example of adding a method wrapper with wrapper-specific notice.

    /// Wrapper for [`QQuickItem::setFlag(QQuickItem::Flag flag, bool enabled = true)`][method] method.
    ///
    /// # Wrapper-specific behavior
    ///
    /// The `enabled` argument is always set to true.
    ///
    /// [method]: https://doc.qt.io/qt-5/qquickitem.html#setFlag
    fn set_flag(&mut self, flag: QQuickItemFlag) {
        let obj = self.get_cpp_object();
        assert!(!obj.is_null());
        cpp!(unsafe [obj as "QQuickItem *", flag as "QQuickItem::Flag"] {
            obj->setFlag(flag);
        });
    }

    fn appendSample(&mut self, value: f64) {
        self.m_samples.push(value);
        self.m_samplesChanged = true;
        // FIXME! find a better way maybe
        self.set_flag(QQuickItemFlag::ItemHasContents);
        (self as &dyn QQuickItem).update();
    }
}

impl QQuickItem for Graph {
    fn geometry_changed(&mut self, new_geometry: QRectF, old_geometry: QRectF) {
        self.m_geometryChanged = true;
        (self as &dyn QQuickItem).update();
    }

    fn update_paint_node(&mut self, mut node: SGNode<ContainerNode>) -> SGNode<ContainerNode> {
        let rect = (self as &dyn QQuickItem).bounding_rect();

        node.update_static((
            |mut n| -> SGNode<nodes::NoisyNode> {
                nodes::create_noisy_node(&mut n, self);
                if self.m_geometryChanged {
                    nodes::noisy_node_set_rect(&mut n, rect);
                }
                n
            },
            |mut n| -> SGNode<nodes::GridNode> {
                if self.m_geometryChanged {
                    nodes::update_grid_node(&mut n, rect);
                }
                n
            },
            |mut n| {
                if self.m_geometryChanged || self.m_samplesChanged {
                    nodes::create_line_node(&mut n, 10., 0.5, QColor::from_name("steelblue"));
                    nodes::update_line_node(&mut n, rect, &self.m_samples);
                }
                n
            },
            |mut n| {
                if self.m_geometryChanged || self.m_samplesChanged {
                    nodes::create_line_node(
                        &mut n,
                        20.,
                        0.2,
                        QColor::from_rgba_f(0.2, 0.2, 0.2, 0.4),
                    );
                    // Fixme! share the geometry
                    nodes::update_line_node(&mut n, rect, &self.m_samples);
                }
                n
            },
        ));

        self.m_geometryChanged = false;
        self.m_samplesChanged = false;
        node
    }
}

fn main() {
    nodes::init_resources();
    qml_register_type::<Graph>(cstr!("Graph"), 1, 0, cstr!("Graph"));
    let mut view = QQuickView::new();
    view.set_source("qrc:/qml/main.qml".into());
    view.show();
    view.engine().exec();
}
