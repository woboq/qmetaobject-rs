// #![feature(trace_macros)] trace_macros!(true);

use std::convert::From;
// use std::convert::Into;
// use std::ops::Deref;
use std::default::Default;
// use std::thread;
use std::cell::{RefCell};
use std::rc::{Rc,Weak};
use std::ops::DerefMut;

type WeakPropertyRef = Weak<PropertyBase>;

trait PropertyBase {
    fn update<'a>(&'a self, dep : Weak<PropertyBase + 'a>);
    fn add_dependency(&self, dep: WeakPropertyRef);
    fn update_dependencies(&self);
}


thread_local!(static CURRENT_PROPERTY: RefCell<Option<WeakPropertyRef>> = Default::default());

#[derive(Default)]
struct PropertyImpl<'a, T> {
    value: RefCell<T>,
    binding : RefCell<Option<Box<Fn()->T + 'a>>>,
    dependencies : RefCell<Vec<WeakPropertyRef>>
}
impl<'a, T> PropertyBase for PropertyImpl<'a, T>  {
    fn update<'b>(&'b self, dep : Weak<PropertyBase + 'b>) {
        if let Some(ref f) = *self.binding.borrow() {
            let mut old = Some(unsafe {
                std::mem::transmute::<Weak<PropertyBase + 'b>, Weak<PropertyBase + 'static>>(dep)
            });

            CURRENT_PROPERTY.with(|cur_dep| {
                let mut m = cur_dep.borrow_mut();
                std::mem::swap(m.deref_mut(), &mut old);
            });
            *self.value.borrow_mut() = f();
            CURRENT_PROPERTY.with(|cur_dep| {
                let mut m = cur_dep.borrow_mut();
                std::mem::swap(m.deref_mut(), &mut old);
            });

            self.update_dependencies();
        }
    }
    fn add_dependency(&self, dep :WeakPropertyRef) {
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
    pub fn from_binding<F : Fn()->T + 'a>(f : F) ->Property<'a, T> {
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
    pub fn set_binding<F : Fn()->T + 'a>(&self, f : F) {
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


//#[allow(non_camel_case_types)]
pub mod anchors {
    use super::{Property, Geometry};
    use std::marker::PhantomData;
    pub enum BeginTag {}
    pub enum EndTag {}
    pub enum CenterTag {}
    pub enum SizeTag {}

    pub struct AnchorElement<'a, Tag : 'a, F : Fn() -> f64 + 'a> {
        f : F,
        _phantom: PhantomData<&'a Tag>
    }
    pub trait AnchorCanAdd<'a, Tag, F> { type Output; fn add(self, F) -> Self::Output; }
    impl<'a, Tag : 'a, F> AnchorCanAdd<'a, Tag, F> for () where F : Fn() -> f64 + 'a {
        type Output = AnchorElement<'a, Tag, F>;
        fn add(self, f: F) -> Self::Output {
            AnchorElement{ f:f , _phantom : PhantomData::default()  }
        }
    }
    macro_rules! declare_AnchorCanAdd {
        ($From:ident => $To:ident) => {
            impl<'a, F1, F2> AnchorCanAdd<'a, $To, F2> for AnchorElement<'a, $From, F1>
                    where F1 : Fn() -> f64 + 'a, F2 : Fn() -> f64 + 'a {
                type Output = (AnchorElement<'a, $From, F1>, AnchorElement<'a, $To, F2>);
                fn add(self, f: F2) -> Self::Output {
                    (self, AnchorElement{ f:f , _phantom : PhantomData::default()  })
                }
            }
        };
        ($From:ident <= $To:ident) => {
            impl<'a, F1, F2> AnchorCanAdd<'a, $To, F2> for AnchorElement<'a, $From, F1>
                    where F1 : Fn() -> f64 + 'a, F2 : Fn() -> f64 + 'a {
                type Output = (AnchorElement<'a, $To, F2>, AnchorElement<'a, $From, F1>);
                fn add(self, f: F2) -> Self::Output {
                    (AnchorElement{ f:f , _phantom : PhantomData::default()}, self)
                }
            }
        };
        // entry point
        ([$($before:ident)* @ $cursor:ident $($tail:ident)*]) => {
            $(declare_AnchorCanAdd!{$cursor => $tail})*
            $(declare_AnchorCanAdd!{$cursor <= $before})*
            // continue (move the cursor
            declare_AnchorCanAdd!{[$($before)* $cursor @ $($tail)*] }
        };
        ([$($before:ident)* @ ]) => { };
    }
    declare_AnchorCanAdd!{[@ BeginTag EndTag CenterTag SizeTag]}

    pub trait AnchorApplyGeometry<'a> {
        fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>);
    }
    impl<'a, F : Fn() -> f64 + 'a> AnchorApplyGeometry<'a> for AnchorElement<'a, BeginTag, F> {
        fn apply_geometry(self, begin: &Property<'a, f64>, _size: &Property<'a, f64>) {
            begin.set_binding(self.f);
        }
    }
    impl<'a, F : Fn() -> f64 + 'a> AnchorApplyGeometry<'a> for AnchorElement<'a, EndTag, F> {
        fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>) {
            let ws = size.as_weak();
            begin.set_binding(move || (self.f)() - ws.get().unwrap() );
        }
    }
    impl<'a> AnchorApplyGeometry<'a> for () {
        fn apply_geometry(self, _begin: &Property<'a, f64>, _size: &Property<'a, f64>) { }
    }



/*
    struct Anchor_None {}
    struct Anchor_Begin<F : Fn() -> f64>(F);
    struct Anchor_End<F : Fn() -> f64>(F);
    struct Anchor_Center<F : Fn() -> f64>(F);
    struct Anchor_Size<F : Fn() -> f64>(F);*/

    pub struct Anchor<Horiz, Vert> {
        h : Horiz,
        v : Vert,
    }
    macro_rules! declare_AnchorFunc {
        ($hname:ident, $vname:ident , $Tag:ident) => {
            pub fn $hname<'a, F : Fn()->f64 + 'a >(self, f : F)
                    -> Anchor<<Horiz as AnchorCanAdd<'a, $Tag, F>>::Output, Vert>
                    where Horiz : AnchorCanAdd<'a, $Tag, F>
            { Anchor { h: self.h.add(f), v: self.v } }
            pub fn $vname<'a, F : Fn()->f64 + 'a>(self, f : F)
                    -> Anchor<Horiz, <Vert as AnchorCanAdd<'a, $Tag, F>>::Output>
                    where Vert : AnchorCanAdd<'a, $Tag, F>
            { Anchor { h: self.h, v: self.v.add(f) } }
        };
    }
    impl<Horiz, Vert> Anchor<Horiz, Vert> {
        declare_AnchorFunc!{left, top, BeginTag}
        declare_AnchorFunc!{right, bottom, EndTag}
        declare_AnchorFunc!{horizontal_center, vertical_center, CenterTag}
        declare_AnchorFunc!{width, height, SizeTag}

        pub fn apply_geometry<'a>(self, g: &Geometry<'a>)
                where Horiz : AnchorApplyGeometry<'a>, Vert : AnchorApplyGeometry<'a> {
            self.h.apply_geometry(& g.x, & g.width);
            self.v.apply_geometry(& g.y, & g.height);
        }
        //pub fn apply_geometry<'a>(self, g: &Geometry<'a>) {}

    }
    pub fn new_anchor() -> Anchor<(), ()> { Anchor { h: (), v: () } }

    #[test]
    fn test_anchor() {
        {
            let g = Geometry::default();
            {
                //let a = new_anchor().right(|| 56.); //.left(|| 78.).top(|| 52.- 11.);
                let a = new_anchor().left(|| 78.).bottom(|| 52.);
                a.apply_geometry(& g);
            }
            g.width.set(12.);
            g.height.set(11.);
            assert_eq!(g.left(), 78.);
            assert_eq!(g.right(), 78. + 12.);
            assert_eq!(g.top(), 52. - 11.);
            assert_eq!(g.bottom(), 52.);

        }
    }


}

/*


mod items {

/*trait ItemLike {
    fn geometry(&mut self) -> &mut Geometry;
    fn add_child(&mut self, child: Rc<Item>)
}

struct Item {
    geometry : Geometry,
}
*/

trait GeometryItem {
    fn geometry(&mut self) -> &mut Geometry;
}

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
}


struct ColumnLayout {
    geometry : Geometry,
    children: Vec<Rc<GeometryItem>>
}
impl GeometryItem for ColumnLayout { fn geometry(&mut self) -> &mut Geometry { &mut self.geometry } }
impl ColumnLayout {
    fn add_child(&mut self, child : Rc<GeometryItem>) {
        children.push(child);
    }
}









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


} // mod items

*/
