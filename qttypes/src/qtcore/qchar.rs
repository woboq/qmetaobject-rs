/// Bindings for [`QChar::UnicodeVersion`][enum] enum.
///
/// [enum]: https://doc.qt.io/qt-5/qchar.html#UnicodeVersion-enum
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum UnicodeVersion {
    Unicode_Unassigned = 0,
    Unicode_1_1 = 1,
    Unicode_2_0 = 2,
    Unicode_2_1_2 = 3,
    Unicode_3_0 = 4,
    Unicode_3_1 = 5,
    Unicode_3_2 = 6,
    Unicode_4_0 = 7,
    Unicode_4_1 = 8,
    Unicode_5_0 = 9,
    Unicode_5_1 = 10,
    Unicode_5_2 = 11,
    Unicode_6_0 = 12,
    Unicode_6_1 = 13,
    Unicode_6_2 = 14,
    Unicode_6_3 = 15,
    Unicode_7_0 = 16,
    Unicode_8_0 = 17,
    #[cfg(qt_5_11)]
    Unicode_9_0 = 18,
    #[cfg(qt_5_11)]
    Unicode_10_0 = 19,
    #[cfg(qt_5_15)]
    Unicode_11_0 = 20,
    #[cfg(qt_5_15)]
    Unicode_12_0 = 21,
    #[cfg(qt_5_15)]
    Unicode_12_1 = 22,
    #[cfg(qt_5_15)]
    Unicode_13_0 = 23,
}
