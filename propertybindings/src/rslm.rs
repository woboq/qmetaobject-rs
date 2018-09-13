
//inspired by on https://www.reddit.com/r/rust/comments/6i6cfl/macro_rules_make_it_hard_to_destruct_rust_structs/
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
            $( pub $field : $crate::properties::Property<'a, $typ> ),*
        }
        /*impl<'a> Default for $name<'a> {
            fn default() -> Self {
                Self {
                    $( $field:  Default::default() /*rsml!{ @decide_field_default $( $value )* }*/ ),*
                }
            }
        }*/
        impl<'a> $name<'a> {
            pub fn new() -> ::std::rc::Rc<Self> {
                let r = ::std::rc::Rc::new(Self { $( $field: rsml!{@parse_default $($value)*} ),* });
                $(rsml!{ @init_field r, $name, $field, $($value)* })*
                r
            }
        }
    };

    //(@init_field $r:ident, $field:ident, = |$s:ident| $bind:expr) => {}
    (@init_field $r:ident, $name:ident, $field:ident $(. $field_cont:ident)* , $bind:expr) => {
        {
            let wr = ::std::rc::Rc::downgrade(&$r);
            #[allow(unused_variables)]
            #[allow(non_snake_case)]
            $r.$field $(. $field_cont)* .set_binding(move || { let $name = wr.upgrade().unwrap(); $bind });
            /*$r.$field $(. $field_cont)* .set_binding((stringify!($name::$field $(. $field_cont)*).to_owned(),
                move || Some({ let $name = wr.upgrade()?; $bind })));*/
        }
    };
    (@init_field $r:ident, $name:ident, $field:ident $(. $field_cont:ident)* ,) => { };
    //(@init_field $r:ident, $field:ident, = $vale:expr) => { };

    //(@parse_default = || $bind:expr) => { Property::from_binding(||$bind) };
    //(@parse_default = $value:expr) => { Property::from($value) };
    (@parse_default $($x:tt)*) => { Default::default() };


    // Initialize an object
    ($name:ident { $($rest:tt)* } ) => {
        rsml!{@parse_as_initialize name: $name, fields: [], sub_items: [], $($rest)* }
    };

    //
    (@parse_as_initialize name: $name:ident, fields: [$($field:ident $(. $field_cont:ident)* : $value:expr ,)*], sub_items: [$($sub_items:tt)*], ) => { {
        let r = $name::new();
        $(rsml!{ @init_field r, $name, $field $(. $field_cont)* , $value })*
        $(rsml!{ @init_sub_items r, $sub_items})*
        r
    } };
    (@parse_as_initialize name: $name:ident, fields: [$($fields:tt)*], sub_items: $sub_items:tt, $field:ident $(. $field_cont:ident)* : $value:expr, $($rest:tt)* ) => {
        rsml!{@parse_as_initialize name: $name, fields: [$($fields)* $field $(. $field_cont)* : $value , ],
            sub_items: $sub_items, $($rest)* }
    };
    (@parse_as_initialize name: $name:ident, fields: [$($fields:tt)*], sub_items: $sub_items:tt, $field:ident $(. $field_cont:ident)* : $value:expr ) => {
        rsml!{@parse_as_initialize name: $name, fields: [$($fields)* $field $(. $field_cont)* : $value , ],
            sub_items: $sub_items, }
    };
    (@parse_as_initialize name: $name:ident, fields: $fields:tt, sub_items: [$($sub_items:tt)*], $nam:ident { $($inner:tt)* }  $($rest:tt)* ) => {
        rsml!{@parse_as_initialize name: $name, fields: $fields, sub_items: [$($sub_items)* { $nam $($inner)* } ], $($rest)* }
    };

    (@init_sub_items $r:ident, { $name:ident $($inner:tt)* } ) => {
        $r.add_child(rsml!{ $name { $($inner)* } });
    };

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
