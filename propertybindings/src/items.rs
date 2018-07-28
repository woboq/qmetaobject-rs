use super::*;
use std::rc::{Rc};
use std::cell::{RefCell};

use qmetaobject::scenegraph::{SGNode,ContainerNode,RectangleNode};
use qmetaobject::{QColor, QQuickItem, QRectF};

pub trait GeometryItem<'a> {
    fn geometry(&self) -> &Geometry<'a>;
    fn update_paint_node(&self, node : SGNode<ContainerNode>, _item: &QQuickItem) -> SGNode<ContainerNode> { node }
}

#[derive(Default)]
pub struct Item<'a> {
    geometry : Geometry<'a>,
}

impl<'a> GeometryItem<'a> for Item<'a> {
    fn geometry(&self) -> &Geometry<'a> {
        return &self.geometry;
    }
}

impl<'a> Item<'a> {
    pub fn new() -> Rc<Self> { Default::default() }
}

#[derive(Default)]
pub struct ColumnLayout<'a> {
    geometry : Geometry<'a>,
    children: RefCell<Vec<Rc<GeometryItem<'a>>>>
}
impl<'a> GeometryItem<'a> for ColumnLayout<'a> {
    fn geometry(&self) -> &Geometry<'a> { &self.geometry }

    fn update_paint_node(&self, mut node : SGNode<ContainerNode>, item: &QQuickItem) -> SGNode<ContainerNode>
    {
        node.update_dynamic(self.children.borrow().iter(),
            |i, n| i.update_paint_node(n, item) );
        node
    }

}
impl<'a> ColumnLayout<'a> {
    pub fn new() -> Rc<Self> { Default::default() }

    pub fn add_child(&self, child : Rc<GeometryItem<'a>>) {
        self.children.borrow_mut().push(child);
        self.relayout();
    }

    fn relayout(&self) {
        let children : Vec<_> = self.children.borrow().iter().map(|x| Rc::downgrade(x)).collect();
        let children_ = children.clone();
        self.geometry.height.set_binding(move || children.iter().map(
            |x| x.upgrade().map_or(0., |y| y.geometry().height())).sum());
        self.geometry.width.set_binding(move|| children_.iter().map(
            |x| x.upgrade().map_or(0., |y| y.geometry().width())).fold(0., f64::max));
        for x in self.children.borrow().windows(2) {
            let a = Rc::downgrade(&x[0]);
            anchors::new_anchor().left(||0.).top(Some (move|| Some(a.upgrade()?.geometry().bottom()))).apply_geometry(x[1].geometry());
        }
    }
}

#[test]
fn test_layout() {
    let lay = Rc::new(ColumnLayout::default());
    lay.add_child(rsml!{
        Item {
            geometry.width : 150.,
            geometry.height : 100.,
        }
    });
    assert_eq!(lay.geometry.width(), 150.);
    assert_eq!(lay.geometry.height(), 100.);
    let middle = rsml!{
        Item {
            geometry.width : 110.,
            geometry.height : 90.,
        }
    };
    lay.add_child(middle.clone());
    lay.add_child(rsml!{
        Item {
            geometry.width : 190.,
            geometry.height : 60.,
        }
    });
    assert_eq!(lay.geometry.width(), 190.);
    assert_eq!(lay.geometry.height(), 100. + 90. + 60.);

    middle.geometry.width.set(200.);
    middle.geometry.height.set(50.);

    assert_eq!(lay.geometry.width(), 200.);
    assert_eq!(lay.geometry.height(), 100. + 50. + 60.);

    assert_eq!(lay.geometry.height(), lay.children.borrow()[2].geometry().bottom());

}


#[derive(Default)]
pub struct Rectangle<'a> {
    geometry : Geometry<'a>,
    color: Property<'a, QColor>
}
impl<'a> GeometryItem<'a> for Rectangle<'a> {
    fn geometry(&self) -> &Geometry<'a> { &self.geometry }
    fn update_paint_node(&self, mut node : SGNode<ContainerNode>, item: &QQuickItem) -> SGNode<ContainerNode>
    {
        node.update_static(
        ((|mut n : SGNode<RectangleNode>| -> SGNode<RectangleNode> {
            n.create(item);
            n.set_color(self.color.get());
            let g = self.geometry();
            n.set_rect(QRectF { x: g.left(), y:g.top(), width: g.width(), height: g.height()  });
            n
        })

        ));
        node
    }

}
impl<'a> Rectangle<'a> {
    pub fn new() -> Rc<Self> { Default::default() }
}



/*


/*
rsml! {
    struct Button {
        #geometry,
        text: String,
        clicked: Event
    }
}

rsml! {
    struct Button {
        #geometry,
        text: String,
        clicked: @event
    }
}*/


rsml! {
// #[derive(Item)]
// struct MyWindow {
//    geometry: Geometry,
    ColumnLayout {
//        geometry: MyWindow.geometry
        Button {
           text: "+",
           clicked => { label.text = label.text.parse() + 1;  }
        }
        let label = Label {
           text: "0",
        }
        Button {
           text: "-",
           clicked => { label.text = label.text.parse() - 1;  }
        }
    }
// }
}

*/

//} // mod items


use qmetaobject::{QObject};


cpp!{{
#include <QtQuick/QQuickItem>
}}

#[derive(QObject, Default)]
struct QuickItem<'a> {
    base: qt_base_class!(trait QQuickItem),
    node : Option<Rc<GeometryItem<'a> + 'a>>,
    init: qt_property!(bool; WRITE set_init)
}
impl<'a> QuickItem<'a> {

    pub fn set_init(&mut self, _: bool) {
        let i : Rc<GeometryItem<'a>> = rsml!(
            Rectangle    {
                geometry.width : 110.,
                geometry.height : 90.,
                color: QColor::from_name("blue"),
            }
        );
        self.set_node(i);
    }

    pub fn set_node(&mut self, node: Rc<GeometryItem<'a> + 'a>) {
        self.node = Some(node);
        let obj = self.get_cpp_object();
        assert!(!obj.is_null());
        cpp!(unsafe [obj as "QQuickItem*"] { obj->setFlag(QQuickItem::ItemHasContents); })
    }
}


impl<'a> QQuickItem for QuickItem<'a>
{
    fn update_paint_node(&mut self, mut node : SGNode<ContainerNode> ) -> SGNode<ContainerNode> {
        if let Some(ref i) = self.node {
            node = i.update_paint_node(node, self);
        }
        node
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        use qmetaobject::*;
        use super::QuickItem;
        qml_register_type::<QuickItem>(cstr!("MyItem"), 1, 0, cstr!("MyItem"));
        let mut engine = QmlEngine::new();
        engine.load_data(r#"
import QtQuick 2.0
import QtQuick.Window 2.0
import MyItem 1.0

Window {
    width: 800
    height: 400
    visible: true

    Rectangle {
        anchors.fill: parent
        anchors.margins: 100
        color: "red"
        border.color: "black"
        border.width: 2
    }

    MyItem {
        anchors.fill: parent
        anchors.margins: 100
        init: true
    }

}



        "#.into());
        engine.exec();
    }

}
