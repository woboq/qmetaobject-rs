
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
        rsml!{@find_all_id (parse_as_initialize_start {$name, $($rest)*}) [] $name => $($rest)* }
    };

    (@parse_as_initialize_start {$name:ident, $($rest:tt)*} $ids:tt ) => {
        rsml!{@parse_as_initialize (parse_as_initialize_end { $name, $ids }), fields: [], sub_items: [], id: [], $($rest)* }
    };



    (@parse_as_initialize_end { $name:ident, [$(($ids:tt $ids_ty:ident))*] } fields: $fields:tt, sub_items: $sub_items:tt, id: $id:tt) => { {
        #[derive(Default)]
        struct IdsContainer<'a> {
            $($ids: ::std::rc::Weak<$ids_ty<'a>> ,)*
            _phantom: ::std::marker::PhantomData<&'a u32>,
        }
        #[allow(unused_variables)]
        let container = std::rc::Rc::new(::std::cell::RefCell::new(IdsContainer::default()));
        let (r, init) = rsml!{@init_sub_items_end {$name container [$($ids)*]} fields: $fields, sub_items: $sub_items, id: $id};
        init();
        r
    } };

    (@parse_as_initialize ($callback:ident $callback_data:tt), fields: $fields:tt, sub_items: $sub_items:tt, id: $id:tt, ) => {
        rsml!{@$callback $callback_data fields: $fields, sub_items: $sub_items, id: $id}
    };

    (@parse_as_initialize $callback:tt, fields: [$($fields:tt)*], sub_items: $sub_items:tt, id: $id:tt, $field:ident $(. $field_cont:ident)* : $value:expr, $($rest:tt)* ) => {
        rsml!{@parse_as_initialize $callback, fields: [$($fields)* $field $(. $field_cont)* : $value , ],
            sub_items: $sub_items, id: $id, $($rest)* }
    };
    (@parse_as_initialize $callback:tt, fields: [$($fields:tt)*], sub_items: $sub_items:tt, id: $id:tt, $field:ident $(. $field_cont:ident)* : $value:expr ) => {
        rsml!{@parse_as_initialize $callback, fields: [$($fields)* $field $(. $field_cont)* : $value , ],
            sub_items: $sub_items, id: $id, }
    };
    (@parse_as_initialize $callback:tt, fields: $fields:tt, sub_items: [$($sub_items:tt)*], id: $id:tt, $nam:ident { $($inner:tt)* }  $($rest:tt)* ) => {
        rsml!{@parse_as_initialize $callback, fields: $fields, sub_items: [$($sub_items)* { $nam $($inner)* } ], id: $id, $($rest)* }
    };
    (@parse_as_initialize $callback:tt, fields: $fields:tt, sub_items: $sub_items:tt, id: [], @id: $id:ident, $($rest:tt)* ) => {
        rsml!{@parse_as_initialize $callback, fields: $fields, sub_items: $sub_items, id: [$id], $($rest)* }
    };
    (@parse_as_initialize $callback:tt, fields: $fields:tt, sub_items: $sub_items:tt, id: $id:tt, , $($rest:tt)* ) => {
        rsml!{@parse_as_initialize $callback, fields: $fields, sub_items: $sub_items, id: $id, $($rest)* }
    };

    (@init_sub_items $r:ident $container:ident $ids:tt, { $name:ident $($inner:tt)* } ) => {
        //        $r.add_child(rsml!{ $name { $($inner)* } });
        let (r, init) = rsml!{@parse_as_initialize (init_sub_items_end { $name $container $ids}), fields: [], sub_items: [], id: [],  $($inner)*};
        $r.add_child(r);
        init
    };

    (@init_sub_items_end { $name:ident $container:ident $ids:tt } fields: [$($field:ident $(. $field_cont:ident)* : $value:expr ,)*], sub_items: [$($sub_items:tt)*], id: [$($id:tt)*]) => { {
        let r = <$name>::new();
        $( $container.borrow_mut().$id = std::rc::Rc::downgrade(&r); )*
        let init = || {};
        $(let i = { rsml!{ @init_sub_items r $container $ids, $sub_items} }; let init = move || { init(); i(); };)*
        #[allow(unused_variables)]
        let container = $container.clone();
        (r.clone(),  move || {init();  $(rsml!{ @init_field_with_ids r, container, $ids, $field $(. $field_cont)* , $value })* })
    } };


    // find all the id then call: rsml!{@callback $callback_data [$($ids)*] }
    (@find_all_id ($callback:ident $callback_data:tt) [$($ids:tt)*] $parent_type:ident => /*$($rest:tt)* */) => {
         rsml!{@$callback $callback_data [$($ids)*] }
    };
    (@find_all_id $callback:tt [$($ids:tt)*] $parent_type:ident => @id: $id:ident $($rest:tt)*) => {
         rsml!{@find_all_id $callback [$($ids)* ($id $parent_type)] $parent_type => $($rest)* }
    };
    (@find_all_id $callback:tt [$($ids:tt)*] $old_parent_type:path => # $parent_type:ident $($rest:tt)*) => {
         rsml!{@find_all_id $callback [$($ids)*] $parent_type => $($rest)* }
    };
    (@find_all_id $callback:tt [$($ids:tt)*] $old_parent_type:path => $parent_type:ident { $($ct:tt)* } $($rest:tt)*) => {
         rsml!{@find_all_id $callback [$($ids)*] $parent_type => $($ct)* # $old_parent_type $($rest)* }
    };
    (@find_all_id $callback:tt [$($ids:tt)*] $parent_type:ident => $_x:tt $($rest:tt)*) => {
         rsml!{@find_all_id $callback [$($ids)*] $parent_type => $($rest)* }
    };


    (@init_field_with_ids $r:ident, $container:ident, [$($id:ident)*], $field:ident $(. $field_cont:ident)* , $bind:expr) => {
        {
            #[allow(unused_variables)]
            let container = $container.clone();
            #[allow(unused_variables)]
            #[allow(non_snake_case)]
            $r.$field $(. $field_cont)* .set_binding(move || { $(let $id = container.borrow().$id.upgrade().unwrap();)* $bind });
            /*$r.$field $(. $field_cont)* .set_binding((stringify!($name::$field $(. $field_cont)*).to_owned(),
                move || Some({ let $name = wr.upgrade()?; $bind })));*/
        }
    };
    (@init_field_with_ids $r:ident, $container:ident, $ids:tt, $field:ident $(. $field_cont:ident)* ,) => { };

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

/*
fn foo() {
rsml!(
            ColumnLayout {
                Container {
                    Rectangle { color: QColor::from_name("grey") }
                    Text {
                        text: "-".into(),
                        vertical_alignment: alignment::VCENTER,
                        horizontal_alignment: alignment::HCENTER,
                    }
                    MouseArea {
                        @id: mouse1,
                        on_clicked: model1.counter.set(model1.counter.get() - 1)
                    }
                }
                Text {
                    text: model.counter.get().to_string().into(),
                    vertical_alignment: alignment::VCENTER,
                    horizontal_alignment: alignment::HCENTER,
                }
                Container {
                    Rectangle { color: QColor::from_name("grey") }
                    Text {
                        text: "+".into(),
                        vertical_alignment: alignment::VCENTER,
                        horizontal_alignment: alignment::HCENTER,
                    }
                    MouseArea {
                        @id: mouse2,
                        on_clicked: model2.counter.set(model2.counter.get() + 1)
                    }
                }
            }
        );
}*/
