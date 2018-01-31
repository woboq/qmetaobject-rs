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
}
impl<'a, T : Default> From<T> for Property<'a, T> {
    fn from(t: T) -> Self {
        Property{ d: Rc::new(PropertyImpl{ value : RefCell::new(t), ..Default::default() }) }
    }
}


//based on https://www.reddit.com/r/rust/comments/6i6cfl/macro_rules_make_it_hard_to_destruct_rust_structs/

macro_rules! rsml {
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
            $( /*$fvis*/ $field : Property<'a, $typ> ),*
        }
        /*impl<'a> Default for $name<'a> {
            fn default() -> Self {
                Self {
                    $( $field:  Default::default() /*rsml!{ @decide_field_default $( $value )* }*/ ),*
                }
            }
        }*/
        impl<'a> $name<'a> {
            fn new() -> Rc<Self> {
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
            $r.$field.set_binding(move || { let $name = wr.upgrade().unwrap(); $bind });
        }
    };
    (@init_field $r:ident, $name:ident, $field:ident,) => { };
    //(@init_field $r:ident, $field:ident, = $vale:expr) => { };

    //(@parse_default = || $bind:expr) => { Property::from_binding(||$bind) };
    //(@parse_default = $value:expr) => { Property::from($value) };
    (@parse_default $($x:tt)*) => { Default::default() };
}






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


    impl<'a> Rectangle<'a> {
        fn new()->Self {
            Rectangle  { ..Default::default() }
        }
    }




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




}
