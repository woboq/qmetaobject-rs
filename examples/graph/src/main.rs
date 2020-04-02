#![allow(non_snake_case)]
#![allow(unused_variables)]
#[macro_use]
extern crate qmetaobject;
use qmetaobject::scenegraph::*;
use qmetaobject::*;
#[macro_use]
extern crate cstr;
#[macro_use]
extern crate cpp;

mod nodes;

#[derive(Default, QObject)]
struct Graph {
    base: qt_base_class!(trait QQuickItem),

    m_samples: Vec<f64>,
    m_samplesChanged: bool,
    m_geometryChanged: bool,

    appendSample: qt_method!(fn(&mut self, value: f64)),
    removeFirstSample: qt_method!(fn removeFirstSample(&mut self) {
        self.m_samples.drain(0..1);
        self.m_samplesChanged = true;
        (self as &dyn QQuickItem).update();
    }),
}

impl Graph {
    fn appendSample(&mut self, value: f64) {
        self.m_samples.push(value);
        self.m_samplesChanged = true;
        // FIXME! find a better way maybe
        let obj = self.get_cpp_object();
        assert!(!obj.is_null());
        cpp!(unsafe [obj as "QQuickItem*"] { obj->setFlag(QQuickItem::ItemHasContents); });
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
    qml_register_type::<Graph>(cstr!("Graph"), 1, 0, cstr!("Graph"));
    let mut view = QQuickView::new();
    view.set_source(format!("{}/src/main.qml", env!("CARGO_MANIFEST_DIR")).into());
    view.show();
    view.engine().exec();
}
