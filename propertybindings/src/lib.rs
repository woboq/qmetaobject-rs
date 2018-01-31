use std::convert::From;
// use std::convert::Into;
// use std::ops::Deref;
use std::default::Default;
use std::thread;
use std::cell::{RefCell, Ref};
use std::rc::{Rc,Weak};
use std::ops::DerefMut;



/*
#[derive(Default)]
pub struct Property<T, Holder> {
    value: T,
    func : Option<Box<Fn(&Holder)->T>>

}

impl<T,Holder> From<T> for Property<T,Holder> {
    fn from(t: T) -> Self { Property{ value :t } }
}
impl<T, Holder> Deref for Property<T,Holder> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}*/

type WeakPropertyRef = Weak<PropertyBase>;

trait PropertyBase {
    fn update(&self, dep : WeakPropertyRef);
    fn add_dependency(&self, dep: WeakPropertyRef);
    fn update_dependencies(&self);
}


thread_local!(static CURRENT_PROPERTY: RefCell<Option<WeakPropertyRef>> = Default::default());
//thread_local!(static CURRENT_DEPENDENCIES: RefCell<Vec<WeakPropertyRef>> =  Default::default());

#[derive(Default)]
struct PropertyImpl<'a, T> {
    value: RefCell<T>,
    binding : RefCell<Option<Box<Fn()->T + 'a>>>,
    dependencies : RefCell<Vec<WeakPropertyRef>>
}
impl<'a, T> PropertyBase for PropertyImpl<'a, T>  {
    fn update(&self, dep : WeakPropertyRef) {
        if let Some(ref f) = *self.binding.borrow() {
            let mut old = Some(dep);
            CURRENT_PROPERTY.with(|cur_dep| {
                let mut m = cur_dep.borrow_mut();
                std::mem::swap(m.deref_mut(), &mut old);
            });
            *self.value.borrow_mut() = f();
            CURRENT_PROPERTY.with(|cur_dep| {
                let mut m = cur_dep.borrow_mut();
                std::mem::swap(m.deref_mut(), &mut old);
            });

            /*
            CURRENT_DEPENDENCIES.with(|cur_dep| {
                let mut m = cur_dep.borrow_mut();
                std::mem::swap(m.deref_mut(), &mut dep);
            });*/
            //self.dependencies = dep;

            //notify dependencies
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
pub struct Property<'a, T> {
    d : Rc<PropertyImpl<'a, T>>
}
impl<'a, T  : Default + Clone + 'static> Property<'a, T>  {
    pub fn from_binding<F : Fn()->T + 'static>(f : F) ->Property<'static, T> {
        let d = Rc::new(PropertyImpl{ binding: RefCell::new(Some(Box::new(f))), ..Default::default()} );
        let w = Rc::downgrade(&d);
        d.update(w);
        Property{ d: d }
    }

    pub fn set(&mut self, t : T) {
        *self.d.binding.borrow_mut() = None;
        *self.d.value.borrow_mut() = t;
        self.d.update_dependencies();
    }

/*
    pub fn borrow<'b>(&'b self) -> Ref<'b, T> {
        self.notify();
        let d = self.d.borrow();
        Ref::map(d, |d| &d.value)
    }*/

    pub fn value(&self) -> T {
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
/*
impl<T> Into<T> for Property<T> {
    fn into(self) -> T { self.value }
}*/

/*
#[derive(Default)]
pub struct Property<'a, T> {
    value: T,
    binding : Option<Box<Fn()->T + 'a>>
//    dependencies : Vec<Property<'a, T>>
}
impl<'a, T> Property<'a, T>  {
    pub fn from_binding<F : Fn()->T + 'a>(f : F) ->Property<'a, T> {
        Property{ value: f(), binding: Some(Box::new(f))  }
    }

}
impl<'a, T> From<T> for Property<'a, T> {
    fn from(t: T) -> Self { Property{ value :t, binding:None } }
}
impl<'a, T> Deref for Property<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
      /*  if let Some(p) = CURRENT_PROPERTY {
            p.add_dependency(self)
        }*/
        &self.d.value
    }
}
*/
/*
impl<T> Default for Property<T> where T: Default {
    fn default() -> Property<T> { Property { value:Default::default() } }
}
*/

 /*
#derive[]
struct Bar {}




struct Foo<'a> {
    x : Option<Bar<'a>>
}
*/

#[cfg(test)]
mod tests {



    #[derive(Default,Clone)]
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
/*
       let mut b = Foo { x:None };
//        let x = || b.x;
//       b.y = Some(move || b.x);
        b.x = Some(Bar { z: RefCell::new(b) } );

*/
       // let f = Foo { ..Default::default() };
      //  b.foo = f;

        //let rec = Rc::new(RefCell::new(Rectangle::new()));
        let mut rec = Rectangle::new();
        rec.width = Property::from(2);
        let tmp = rec.clone();
        rec.height.set(4);
        rec.area = Property::from_binding(move || tmp.width.value() * tmp.height.value());
        //Property::from(4);
        assert_eq!(rec.area.value(), 4*2);
    }
}
