// #![feature(trace_macros)] trace_macros!(true);

use std::convert::From;
// use std::convert::Into;
// use std::ops::Deref;
use std::default::Default;
// use std::thread;
use std::cell::{RefCell, Cell};
use std::rc::{Rc,Weak};
use std::ops::DerefMut;

type WeakPropertyRef = Weak<PropertyBase>;

trait PropertyBase {
    fn update<'a>(&'a self, dep : Weak<PropertyBase + 'a>);
    fn add_dependency(&self, dep: WeakPropertyRef);
    fn update_dependencies(&self);
    fn description(&self) -> String { String::default() }
}

thread_local!(static CURRENT_PROPERTY: RefCell<Option<WeakPropertyRef>> = Default::default());

pub trait PropertyBindingFn<T> {
    fn run(&self) -> Option<T>;
    fn description(&self) -> String { String::default() }
}
impl<F, T> PropertyBindingFn<T> for F where F : Fn()->T {
    fn run(&self) -> Option<T> { Some((*self)()) }
}
impl<F, T> PropertyBindingFn<T> for Option<F> where F : Fn()->Option<T> {
    fn run(&self) -> Option<T> { self.as_ref().and_then(|x|x()) }
}

impl<F, T> PropertyBindingFn<T> for (String, F) where F : Fn()->Option<T> {
    fn run(&self) -> Option<T> { (self.1)() }
    fn description(&self) -> String { (self.0).clone() }
}

#[derive(Default)]
struct PropertyImpl<'a, T> {
    value: RefCell<T>,
    binding : RefCell<Option<Box<PropertyBindingFn<T> + 'a>>>,
    dependencies : RefCell<Vec<WeakPropertyRef>>,
    updating: Cell<bool>,
}
impl<'a, T> PropertyBase for PropertyImpl<'a, T>  {
    fn update<'b>(&'b self, dep : Weak<PropertyBase + 'b>) {
        if let Some(ref f) = *self.binding.borrow() {
            if self.updating.get() {
                panic!("Circular dependency found : {}", self.description());
            }
            self.updating.set(true);

            let mut old = Some(unsafe {
                std::mem::transmute::<Weak<PropertyBase + 'b>, Weak<PropertyBase + 'static>>(dep)
            });

            CURRENT_PROPERTY.with(|cur_dep| {
                let mut m = cur_dep.borrow_mut();
                std::mem::swap(m.deref_mut(), &mut old);
            });
            let mut modified = false;
            if let Some(val) = f.run() {
                *self.value.borrow_mut() = val;
                modified = true;
            }
            CURRENT_PROPERTY.with(|cur_dep| {
                let mut m = cur_dep.borrow_mut();
                std::mem::swap(m.deref_mut(), &mut old);
                //assert!(Rc::ptr_eq(&dep.upgrade().unwrap(), &old.unwrap().upgrade().unwrap()));
            });
            if modified {
                self.update_dependencies();
            }
            self.updating.set(false);
        }
    }
    fn add_dependency(&self, dep :WeakPropertyRef) {
        //println!("ADD DEPENDENCY {} -> {}",  self.description(), dep.upgrade().map_or("NONE".into(), |x| x.description()));
        self.dependencies.borrow_mut().push(dep);
    }

    fn update_dependencies(&self) {
        let mut v = vec![];
        {
            let mut dep = self.dependencies.borrow_mut();
            std::mem::swap(dep.deref_mut(), &mut v);
        }
        for d in &v {
            if let Some(d) = d.upgrade() {
                let w = Rc::downgrade(&d);
                d.update(w);
            }
        }
    }

    fn description(&self) -> String {
        if let Some(ref f) = *self.binding.borrow() {
            f.description()
        } else {
            String::default()
        }
    }
}

#[derive(Default,Clone)]
pub struct WeakProperty<'a, T> {
    d : Weak<PropertyImpl<'a, T>>
}
impl<'a, T  : Default + Clone> WeakProperty<'a, T>  {
    pub fn get(&self) -> Option<T> {
        self.d.upgrade().map(|x| (Property{ d: x}).get())
    }
}

#[derive(Default)]
pub struct Property<'a, T> {
    d : Rc<PropertyImpl<'a, T>>
}
impl<'a, T  : Default + Clone> Property<'a, T>  {
    pub fn from_binding<F : PropertyBindingFn<T> + 'a>(f : F) ->Property<'a, T> {
        let d = Rc::new(PropertyImpl{ binding: RefCell::new(Some(Box::new(f))), ..Default::default()} );
        let w = Rc::downgrade(&d);
        d.update(w);
        Property{ d: d }
    }

    pub fn set(&self, t : T) {
        *self.d.binding.borrow_mut() = None;
        *self.d.value.borrow_mut() = t;
        self.d.update_dependencies();
    }
    pub fn set_binding<F : PropertyBindingFn<T> + 'a>(&self, f : F) {
        *self.d.binding.borrow_mut() = Some(Box::new(f));
        let w = Rc::downgrade(&self.d);
        self.d.update(w);
    }

/*
    pub fn borrow<'b>(&'b self) -> Ref<'b, T> {
        self.notify();
        let d = self.d.borrow();
        Ref::map(d, |d| &d.value)
    }*/

    // FIXME! remove
    pub fn value(&self) -> T {
        self.notify();
        self.d.value.borrow().clone()
    }

    pub fn get(&self) -> T {
        self.notify();
        self.d.value.borrow().clone()
    }

    fn notify(&self) {
        CURRENT_PROPERTY.with(|cur_dep| {
            if let Some(m) = (*cur_dep.borrow()).clone() {
                if !m.upgrade().is_none() {
                    self.d.add_dependency(m);
                }
            }
        });
    }

    fn as_weak(&self) -> WeakProperty<'a, T> {
        WeakProperty{ d: Rc::downgrade(&self.d) }
    }
}
impl<'a, T : Default> From<T> for Property<'a, T> {
    fn from(t: T) -> Self {
        Property{ d: Rc::new(PropertyImpl{ value : RefCell::new(t), ..Default::default() }) }
    }
}

//based on https://www.reddit.com/r/rust/comments/6i6cfl/macro_rules_make_it_hard_to_destruct_rust_structs/
#[macro_export]
macro_rules! rsml {
    // Declare a struct
    ($(#[$($attrs:tt)*])* pub($($vis:tt)*) struct $name:ident {$($body:tt)*}) => {
        rsml! { @parse_fields $(#[$($attrs)*])*, [pub($($vis)*)], $name, $($body)* }
    };
    ($(#[$($attrs:tt)*])* pub struct $name:ident {$($body:tt)*}) => {
        rsml! { @parse_fields $(#[$($attrs)*])*, [pub], $name, $($body)* }
    };
    ($(#[$($attrs:tt)*])* struct $name:ident {$($body:tt)*}) => {
        rsml! { @parse_fields $(#[$($attrs)*])*, [], $name, $($body)* }
    };

    (@parse_fields $(#[$attrs:meta])*, [$($vis:tt)*], $name:ident,
            $(/*$fvis:vis*/ $field:ident : $typ:ty  $(= $value:expr )* ),* $(,)*) => {
        $(#[$attrs])* $($vis)* struct $name<'a> {
            $( pub $field : Property<'a, $typ> ),*
        }
        /*impl<'a> Default for $name<'a> {
            fn default() -> Self {
                Self {
                    $( $field:  Default::default() /*rsml!{ @decide_field_default $( $value )* }*/ ),*
                }
            }
        }*/
        impl<'a> $name<'a> {
            pub fn new() -> Rc<Self> {
                let r = Rc::new(Self { $( $field: rsml!{@parse_default $($value)*} ),* });
                $(rsml!{ @init_field r, $name, $field, $($value)* })*
                r
            }
        }
    };

    //(@init_field $r:ident, $field:ident, = |$s:ident| $bind:expr) => {}
    (@init_field $r:ident, $name:ident, $field:ident, $bind:expr) => {
        {
            let wr = Rc::downgrade(&$r);
            #[allow(unused_variables)]
            #[allow(non_snake_case)]
            $r.$field.set_binding(move || { let $name = wr.upgrade().unwrap(); $bind });
        }
    };
    (@init_field $r:ident, $name:ident, $field:ident,) => { };
    //(@init_field $r:ident, $field:ident, = $vale:expr) => { };

    //(@parse_default = || $bind:expr) => { Property::from_binding(||$bind) };
    //(@parse_default = $value:expr) => { Property::from($value) };
    (@parse_default $($x:tt)*) => { Default::default() };


    // Initialize an object
    ($name:ident {$($field:ident : $value:expr ),* $(,)* } ) => { {
        let r = $name::new();
        $(rsml!{ @init_field r, $name, $field, $value })*
        r
    } };

}



/*


trait Parent {
    fn add_child(this: Rc<Self>, child: Rc<Child>);
}
trait Child {
    fn set_parent(&self, parent: Weak<Child>);
}


macro_rules! rsml_init {
    ($name:ident { $($field:ident = $value:expr ),* $(,)* } ) => {
        let x = $name::new();
        $name = X


    };

    (@parse_fields $(#[$attrs:meta])*, [$($vis:tt)*], $name:ident,
            $(/*$fvis:vis*/ $field:ident : $typ:ty  $(= $value:expr )* ),* $(,)*) => {
        $(#[$attrs])* $($vis)* struct $name<'a> {
            $( pub $field : Property<'a, $typ> ),*
        }

    };

}
*/




#[cfg(test)]
mod tests {



    #[derive(Default)]
    struct Rectangle<'a>  {
        /*
        property<rectangle*> parent = nullptr;
        property<int> width = 150;
        property<int> height = 75;
        property<int> area = [&]{ return calculateArea(width, height); };

        property<std::string> color = [&]{
            if (parent() && area > parent()->area)
            return std::string("blue");
            else
            return std::string("red");
        };*/

        width : Property<'a, u32>,
        height : Property<'a, u32>,
        area : Property<'a, u32>,

    }

/*
    impl<'a> Rectangle<'a> {
        fn new()->Self {
            Rectangle  { ..Default::default() }
        }
    }*/

    use ::*;

    #[test]
    fn it_works() {


        let rec = Rc::new(RefCell::new(Rectangle::default()));
        rec.borrow_mut().width = Property::from(2);
        let wr = Rc::downgrade(&rec);
        rec.borrow_mut().area = Property::from_binding(move || wr.upgrade().map(|wr| wr.borrow().width.value() * wr.borrow().height.value()).unwrap());
        rec.borrow().height.set(4);
        assert_eq!(rec.borrow().area.value(), 4*2);
    }


    rsml!{
        struct Rectangle2 {
            width: u32 = 2,
            height: u32,
            area: u32 = Rectangle2.width.value() * Rectangle2.height.value()
        }
    }

    #[test]
    fn test_rsml() {

        let rec = Rectangle2::new(); // Rc::new(RefCell::new(Rectangle2::default()));
//         let wr = Rc::downgrade(&rec);
//         rec.borrow_mut().area = Property::from_binding(move || wr.upgrade().map(|wr| wr.borrow().width.value() * wr.borrow().height.value()).unwrap());
        rec.height.set(4);
        assert_eq!(rec.area.value(), 4*2);
        rec.height.set(8);
        assert_eq!(rec.area.value(), 8*2);
    }


    #[test]
    fn test_rsml_init() {
        let rec = rsml!{
            Rectangle2 {
                height: Rectangle2.width.value() * 3,
            }
        };
        assert_eq!(rec.area.value(), 3*2*2);
        rec.width.set(8);
        assert_eq!(rec.area.value(), 3*8*8);
    }


/*
    rsml!{
        struct Item {
            width: u32,
            height: u32,
            x: u32,
            y: u32,
        }
    }

    rsml!{
        struct Rectangle {
            base: Rc<Item> = ,
            color: u32 = 0xffffffff,
            border_color: u32,
            border_width: 0
        }
    }

    rsml!{
        struct MyComponent {
            base: Rc<Item>,
            r1: Rc<Rectangle> = rsml_instance!{Rectangle {
                base: rsml_instance!{Item {

            }}
        }
    }
*/




}


#[derive(Default)]
pub struct Geometry<'a> {
    x: Property<'a,f64>,
    y: Property<'a,f64>,
    width: Property<'a,f64>,
    height: Property<'a,f64>,
}
impl<'a> Geometry<'a> {
    pub fn width(&self) -> f64 { self.width.get() }
    pub fn height(&self) -> f64 { self.height.get() }
    pub fn left(&self) -> f64 { self.x.get() }
    pub fn top(&self) -> f64 { self.y.get() }
    pub fn right(&self) -> f64 { self.x.get() + self.width.get() }
    pub fn bottom(&self) -> f64 { self.y.get() + self.height.get() }
    pub fn vertical_center(&self)  -> f64 { self.x.get() + self.width.get() / 2. }
    pub fn horizontal_center(&self)  -> f64 { self.y.get() + self.height.get() / 2. }
}


pub mod anchors;


/*

mod items {

use std::rc::{Rc};
use super::{Geometry};

/*trait ItemLike {
    fn geometry(&mut self) -> &mut Geometry;
    fn add_child(&mut self, child: Rc<Item>)
}

struct Item {
    geometry : Geometry,
}
*/

trait GeometryItem<'a> {
    fn geometry(&self) -> &Geometry<'a>;
}
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


struct ColumnLayout<'a> {
    geometry : Geometry<'a>,
    children: Vec<Rc<GeometryItem<'a>>>
}
impl<'a> GeometryItem<'a> for ColumnLayout<'a> { fn geometry(&self) -> &Geometry<'a> { &self.geometry } }
impl<'a> ColumnLayout<'a> {
    pub fn add_child(&mut self, child : Rc<GeometryItem<'a>>) {
        self.children.push(child);

    }

    fn relayout(&self) {
        let children : Vec<_> = self.children.iter().map(|x| Rc::downgrade(x)).collect();
        self.geometry.width.set_binding(move|| children.iter().map(
            |x| x.upgrade().map_or(0., |y| y.geometry().width())).sum());
    }
}



*/



/*

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
