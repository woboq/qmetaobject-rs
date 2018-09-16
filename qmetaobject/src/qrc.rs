cpp!{{
Q_CORE_EXPORT bool qRegisterResourceData(int, const unsigned char *,
                                         const unsigned char *, const unsigned char *);
}}

/// Macro to embed files and made them available to the Qt resource system
///
/// ```ignore
/// qrc!(my_ressource,
///     "qml" {
///         "main.qml",
///         "Foo.qml" as "foo/Foo.qml",
///      }
/// );
///
/// //...
/// my_resource(); // registers the resource to Qt
/// ```
///
/// corresponds to the .qrc file:
/// ```ignore
///    <RCC>
///        <qresource prefix="/qml">
///            <file>main.qml</file>
///            <file alias="foo/Foo.qml">Foo.qml</file>
///        </qresource>
///    </RCC>
/// ```
///
/// The paths are relative to the location in which cargo runs.
///
/// The macro creates a function that needs to be run in order to register the
/// resource. Calling the function more than once has no effect.
#[macro_export]
macro_rules! qrc {
    ($name:ident, $($rest:tt)* ) => {
        fn $name() {
            #[allow(unused)]
            #[derive(QResource_internal)]
            enum RC { Input = (0, stringify!($($rest)*)).0 }
            register();
        }
    }
}

/// Internal function used from qrc procedural macro.
/// Unsafe because it can crash if the data structure are not proper.
#[doc(ignore)]
pub unsafe fn register_resource_data(
    version: i32,
    tree: &'static [u8],
    names: &'static [u8],
    payload: &'static [u8],
) {
    let tree_ptr = tree.as_ptr();
    let names_ptr = names.as_ptr();
    let payload_ptr = payload.as_ptr();
    cpp!([version as "int", tree_ptr as "const unsigned char*", names_ptr as "const unsigned char*", payload_ptr as "const unsigned char*"] {
        qRegisterResourceData(version, tree_ptr, names_ptr, payload_ptr);
    });
}
