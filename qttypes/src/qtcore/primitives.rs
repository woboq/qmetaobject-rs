/// Bindings for [`qreal`][type] typedef.
///
/// [type]: https://doc.qt.io/qt-5/qtglobal.html#qreal-typedef
#[allow(non_camel_case_types)]
#[cfg(qreal_is_float)]
pub type qreal = f32;

#[allow(non_camel_case_types)]
#[cfg(not(qreal_is_float))]
pub type qreal = f64;
