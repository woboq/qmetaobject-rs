use super::*;
use std::rc::{Rc};
use std::cell::{RefCell};


pub trait GeometryItem<'a> {
    fn geometry(&self) -> &Geometry<'a>;
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
impl<'a> GeometryItem<'a> for ColumnLayout<'a> { fn geometry(&self) -> &Geometry<'a> { &self.geometry } }
impl<'a> ColumnLayout<'a> {
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
