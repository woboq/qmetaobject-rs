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


        let rec = Rc::new(RefCell::new(Rectangle::new()));
        rec.borrow_mut().width = Property::from(2);
        let wr = Rc::downgrade(&rec);
        rec.borrow_mut().area = Property::from_binding(move || wr.upgrade().map(|wr| wr.borrow().width.value() * wr.borrow().height.value()).unwrap());
        rec.borrow().height.set(4);
        assert_eq!(rec.borrow().area.value(), 4*2);
    }
}
