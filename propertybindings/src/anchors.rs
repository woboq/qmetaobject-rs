use super::{Property, Geometry, PropertyBindingFn};
use std::marker::PhantomData;
pub enum BeginTag {}
pub enum EndTag {}
pub enum CenterTag {}
pub enum SizeTag {}

pub struct AnchorElement<'a, Tag : 'a, F : PropertyBindingFn<f64> + 'a> {
    f : F,
    _phantom: PhantomData<&'a Tag>
}
pub trait AnchorCanAdd<'a, Tag, F> { type Output; fn add(self, F) -> Self::Output; }
impl<'a, Tag : 'a, F> AnchorCanAdd<'a, Tag, F> for () where F : PropertyBindingFn<f64> + 'a {
    type Output = AnchorElement<'a, Tag, F>;
    fn add(self, f: F) -> Self::Output {
        AnchorElement{ f:f , _phantom : PhantomData::default()  }
    }
}
macro_rules! declare_AnchorCanAdd {
    ($From:ident => $To:ident) => {
        impl<'a, F1, F2> AnchorCanAdd<'a, $To, F2> for AnchorElement<'a, $From, F1>
                where F1 : PropertyBindingFn<f64> + 'a, F2 : PropertyBindingFn<f64> + 'a {
            type Output = (AnchorElement<'a, $From, F1>, AnchorElement<'a, $To, F2>);
            fn add(self, f: F2) -> Self::Output {
                (self, AnchorElement{ f:f , _phantom : PhantomData::default()  })
            }
        }
    };
    ($From:ident <= $To:ident) => {
        impl<'a, F1, F2> AnchorCanAdd<'a, $To, F2> for AnchorElement<'a, $From, F1>
                where F1 : PropertyBindingFn<f64> + 'a, F2 : PropertyBindingFn<f64> + 'a {
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
impl<'a> AnchorApplyGeometry<'a> for () {
    fn apply_geometry(self, _begin: &Property<'a, f64>, _size: &Property<'a, f64>) { }
}
impl<'a, F : PropertyBindingFn<f64> + 'a> AnchorApplyGeometry<'a> for AnchorElement<'a, BeginTag, F> {
    fn apply_geometry(self, begin: &Property<'a, f64>, _size: &Property<'a, f64>) {
        begin.set_binding(self.f);
    }
}
impl<'a, F : PropertyBindingFn<f64> + 'a> AnchorApplyGeometry<'a> for AnchorElement<'a, EndTag, F> {
    fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>) {
        let ws = size.as_weak();
        begin.set_binding(Some(move || Some(self.f.run()? - ws.get()?) ));
    }
}
impl<'a, F : PropertyBindingFn<f64> + 'a> AnchorApplyGeometry<'a> for AnchorElement<'a, CenterTag, F> {
    fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>) {
        let ws = size.as_weak();
        begin.set_binding(Some(move || Some(self.f.run()? - ws.get()? / 2.) ));
    }
}
impl<'a, F : PropertyBindingFn<f64> + 'a> AnchorApplyGeometry<'a> for AnchorElement<'a, SizeTag, F> {
    fn apply_geometry(self, _begin: &Property<'a, f64>, size: &Property<'a, f64>) {
        size.set_binding(self.f);
    }
}
impl<'a, F1 : PropertyBindingFn<f64> + 'a, F2 : PropertyBindingFn<f64> + 'a> AnchorApplyGeometry<'a>
        for (AnchorElement<'a, BeginTag, F1>, AnchorElement<'a, EndTag, F2>) {
    fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>) {
        begin.set_binding((self.0).f);
        let end = self.1;
        let ws = begin.as_weak();
        size.set_binding(Some(move || Some(end.f.run()? - ws.get()?)));
    }
}
impl<'a, F1 : PropertyBindingFn<f64> + 'a, F2 : PropertyBindingFn<f64> + 'a> AnchorApplyGeometry<'a>
        for (AnchorElement<'a, BeginTag, F1>, AnchorElement<'a, CenterTag, F2>) {
    fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>) {
        begin.set_binding((self.0).f);
        let center = self.1;
        let ws = begin.as_weak();
        size.set_binding(Some(move || Some((center.f.run()? - ws.get()?) * 2.)));
    }
}
impl<'a, F1 : PropertyBindingFn<f64> + 'a, F2 : PropertyBindingFn<f64> + 'a> AnchorApplyGeometry<'a>
        for (AnchorElement<'a, BeginTag, F1>, AnchorElement<'a, SizeTag, F2>) {
    fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>) {
        begin.set_binding((self.0).f);
        size.set_binding((self.1).f);
    }
}
impl<'a, F1 : PropertyBindingFn<f64> + Clone + 'a, F2 : PropertyBindingFn<f64> + Clone + 'a> AnchorApplyGeometry<'a>
        for (AnchorElement<'a, EndTag, F1>, AnchorElement<'a, CenterTag, F2>) {
    fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>) {
        let center = (self.1).f.clone();
        let end = (self.0).f.clone();
        begin.set_binding(Some(move || Some(2. * (self.1).f.run()? - (self.0).f.run()?)));
        size.set_binding(Some(move || Some((end.run()? - center.run()?) * 2.)));
    }
}
impl<'a, F1 : PropertyBindingFn<f64> + 'a, F2 : PropertyBindingFn<f64> + 'a> AnchorApplyGeometry<'a>
        for (AnchorElement<'a, EndTag, F1>, AnchorElement<'a, SizeTag, F2>) {
    fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>) {
        size.set_binding((self.1).f);
        let ws = size.as_weak();
        let end = self.0;
        begin.set_binding(Some(move || Some(end.f.run()? - ws.get()?)));
    }
}
impl<'a, F1 : PropertyBindingFn<f64> + 'a, F2 : PropertyBindingFn<f64> + 'a> AnchorApplyGeometry<'a>
        for (AnchorElement<'a, CenterTag, F1>, AnchorElement<'a, SizeTag, F2>) {
    fn apply_geometry(self, begin: &Property<'a, f64>, size: &Property<'a, f64>) {
        size.set_binding((self.1).f);
        let center = self.0;
        let ws = size.as_weak();
        begin.set_binding(Some(move || Some(center.f.run()? - ws.get()? / 2.)));
    }
}



pub struct Anchor<Horiz, Vert> {
    h : Horiz,
    v : Vert,
}
macro_rules! declare_AnchorFunc {
    ($hname:ident, $vname:ident , $Tag:ident) => {
        pub fn $hname<'a, F : PropertyBindingFn<f64> + 'a >(self, f : F)
                -> Anchor<<Horiz as AnchorCanAdd<'a, $Tag, F>>::Output, Vert>
                where Horiz : AnchorCanAdd<'a, $Tag, F>
        { Anchor { h: self.h.add(f), v: self.v } }
        pub fn $vname<'a, F : PropertyBindingFn<f64> + 'a>(self, f : F)
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

}
pub fn new_anchor() -> Anchor<(), ()> { Anchor { h: (), v: () } }

#[test]
fn test_anchor() {
    {
        let a = new_anchor().left(|| 78.).bottom(|| 52.);
        let g = Geometry::default();
        a.apply_geometry(& g);
        g.width.set(12.);
        g.height.set(11.);
        assert_eq!(g.left(), 78.);
        assert_eq!(g.right(), 78. + 12.);
        assert_eq!(g.top(), 52. - 11.);
        assert_eq!(g.bottom(), 52.);
    }

    {
        let a = new_anchor().left(||78.).bottom(|| 52.).width(|| 12.).height(|| 11.);
        let g = Geometry::default();
        a.apply_geometry(& g);
        assert_eq!(g.left(), 78.);
        assert_eq!(g.right(), 78. + 12.);
        assert_eq!(g.top(), 52. - 11.);
        assert_eq!(g.bottom(), 52.);
    }
}
