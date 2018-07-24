extern crate qmetaobject;
use qmetaobject::*;
use qmetaobject::scenegraph::*;
#[macro_use] extern crate cstr;


#[allow(unused_variables)]
#[allow(non_snake_case)]


#[derive(Default, QObject)]
struct Graph {
    base: qt_base_class!(trait QQuickItem),

    m_samples : Vec<f64>,
    m_samplesChanged : bool,
    m_geometryChanged : bool,

    appendSample : qt_method!(fn appendSample(&mut self, value: f64) {
        self.m_samples.push(value);
	    self.m_samplesChanged = true;
	    //update();
    }),
    removeFirstSample : qt_method!(fn removeFirstSample(&mut self) {
        self.m_samples.drain(0..1);
        self.m_samplesChanged = true;
        //update();
    }),
}


struct LineNode {
    inside: SGNode,
}
impl LineNode {
    fn update_geometry(&mut self, _ : QRectF, _ : &[f64]) {}
}

fn createLineNode(_ : u32, _ : f64, _ : QColor) -> LineNode {
    LineNode{ inside: SGNode{ raw: std::ptr::null_mut() } }
}

impl NodeType for LineNode {
    fn from_node(n: &mut SGNode) -> &mut Self {
        unsafe { std::mem::transmute(n) }
    }
    fn into_node(self) -> SGNode {
        self.inside
    }
}


impl QQuickItem for Graph
{
    fn geometry_changed(&mut self, new_geometry : QRectF, old_geometry : QRectF) {
        self.m_geometryChanged = true;
        (self as &QQuickItem).update();
    }

    fn update_paint_node(&mut self, mut node : SGNode ) {
        let rect = (self as &QQuickItem).bounding_rect();
        node.update_nodes(&[
            &UpdateNode {
                create: (|| createLineNode(10, 0.5, QColor::from_name("steelblue"))),
                update: (|n| { if self.m_geometryChanged || self.m_samplesChanged {
                            n.update_geometry(rect, &self.m_samples);
                        } })
            },
            &UpdateNode {
                create: (|| createLineNode(20, 0.2, QColor::from_rgba_f(0.2,0.2,0.2, 0.4))),
                update: (|n| { if self.m_geometryChanged || self.m_samplesChanged {
                            n.update_geometry(rect, &self.m_samples);
                        } })
            }
        ]);
        /*node.update_node(vec![
            ( (|ctx| createNoisyNode(ctx)) ,
                ( |n| { if self.m_geometryChanged { n.set_rect(rect); } } )),
            ( (|ctx| createGridNode(ctx)) ,
                (|n| { if self.m_geometryChanged { n.set_rect(rect); } } )),
            ( (|_| createLineNode(10, 0.5, QColor::from_name("steelblue"))),
                |n| { if self.m_geometryChanged || self.m_samplesChanged {
                    n.update_geometry(rect, self.m_samples);
                } } ),
            ( (|_| createLineNode(20, 0.2, QColor::from_rgba_f(0.2,0.2,0.2, 0.4))),
                |n| { if self.m_geometryChanged || self.m_samplesChanged {
                    n.update_geometry(rect, self.m_samples);
                } } )

        ]);*/
        self.m_geometryChanged = false;
        self.m_samplesChanged = false;
    }
}

fn main() {
    qml_register_type::<Graph>(cstr!("Graph"), 1, 0, cstr!("Graph"));
    let mut view = QQuickView::new();
    view.set_source("src/main.qml".into());
    view.show();
    view.engine().exec();
}
