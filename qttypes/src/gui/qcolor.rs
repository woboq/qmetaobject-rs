use crate::internal_prelude::*;
use crate::{qreal, QString};

cpp! {{
    #include <QtGui/QColor>
    #include <QtCore/QString>
}}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct QRgb(u32);

impl QRgb {
    pub fn alpha(&self) -> u8 {
        ((self.0 >> 24) & 0x000000ff) as u8
    }
    pub fn red(&self) -> u8 {
        ((self.0 >> 16) & 0x000000ff) as u8
    }
    pub fn green(&self) -> u8 {
        ((self.0 >> 8) & 0x000000ff) as u8
    }
    pub fn blue(&self) -> u8 {
        (self.0 & 0x000000ff) as u8
    }
    pub fn grayscale(&self) -> u8 {
        let value = self.0;
        let ret: i32 = cpp!(unsafe [value as "QRgb"] -> i32 as "int" {
            return qGray(value);
        });
        assert!(ret > 0 && ret <= 255);
        ret as u8
    }
    pub fn premultiply(&self) -> Self {
        let value = self.0;
        cpp!(unsafe [value as "QRgb"] -> QRgb as "QRgb" {
            return qPremultiply(value);
        })
    }
    pub fn unpremultiply(&self) -> Self {
        let value = self.0;
        cpp!(unsafe [value as "QRgb"] -> QRgb as "QRgb" {
            return qUnpremultiply(value);
        })
    }
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        cpp!(unsafe [r as "quint8", g as "quint8", b as "quint8"] -> QRgb as "QRgb" {
            return qRgb(r, g, b);
        })
    }
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        cpp!(unsafe [r as "quint8", g as "quint8", b as "quint8", a as "quint8"] -> QRgb as "QRgb" {
            return qRgba(r, g, b, a);
        })
    }
}

impl From<u32> for QRgb {
    fn from(val: u32) -> QRgb {
        QRgb(val)
    }
}

impl Into<u32> for QRgb {
    fn into(self) -> u32 {
        self.0
    }
}

cpp_class!(
    /// Wrapper around [`QColor`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qcolor.html
    #[derive(Default, Clone, Copy, PartialEq)]
    pub unsafe struct QRgba64 as "QRgba64"
);
impl QRgba64 {
    pub fn alpha(&self) -> u16 {
        cpp!(unsafe [self as "const QRgba64*"] -> u16 as "quint16" {
            return self->alpha();
        })
    }

    pub fn alpha8(&self) -> u8 {
        cpp!(unsafe [self as "const QRgba64*"] -> u8 as "quint8" {
            return self->alpha8();
        })
    }

    pub fn red(&self) -> u16 {
        cpp!(unsafe [self as "const QRgba64*"] -> u16 as "quint16" {
            return self->red();
        })
    }

    pub fn red8(&self) -> u8 {
        cpp!(unsafe [self as "const QRgba64*"] -> u8 as "quint8" {
            return self->red8();
        })
    }

    pub fn green(&self) -> u16 {
        cpp!(unsafe [self as "const QRgba64*"] -> u16 as "quint16" {
            return self->green();
        })
    }

    pub fn green8(&self) -> u8 {
        cpp!(unsafe [self as "const QRgba64*"] -> u8 as "quint8" {
            return self->green8();
        })
    }

    pub fn blue(&self) -> u16 {
        cpp!(unsafe [self as "const QRgba64*"] -> u16 as "quint16" {
            return self->blue();
        })
    }

    pub fn blue8(&self) -> u8 {
        cpp!(unsafe [self as "const QRgba64*"] -> u8 as "quint8" {
            return self->blue8();
        })
    }

    pub fn is_opaque(&self) -> bool {
        cpp!(unsafe [self as "const QRgba64*"] -> bool as "bool" {
            return self->isOpaque();
        })
    }

    pub fn is_transparent(&self) -> bool {
        cpp!(unsafe [self as "const QRgba64*"] -> bool as "bool" {
            return self->isTransparent();
        })
    }

    pub fn premultiplied(&self) -> Self {
        cpp!(unsafe [self as "const QRgba64*"] -> QRgba64 as "QRgba64" {
            return self->premultiplied();
        })
    }

    pub fn unpremultiplied(&self) -> Self {
        cpp!(unsafe [self as "const QRgba64*"] -> QRgba64 as "QRgba64" {
            return self->unpremultiplied();
        })
    }

    pub fn set_alpha(&mut self, alpha: u16) {
        cpp!(unsafe [self as "QRgba64*", alpha as "quint16"] {
            self->setAlpha(alpha);
        })
    }

    pub fn set_red(&mut self, red: u16) {
        cpp!(unsafe [self as "QRgba64*", red as "quint16"] {
            self->setRed(red);
        })
    }

    pub fn set_green(&mut self, green: u16) {
        cpp!(unsafe [self as "QRgba64*", green as "quint16"] {
            self->setGreen(green);
        })
    }

    pub fn set_blue(&mut self, blue: u16) {
        cpp!(unsafe [self as "QRgba64*", blue as "quint16"] {
            self->setBlue(blue);
        })
    }

    pub fn from_rgba(r: u16, g: u16, b: u16, a: u16) -> QRgba64 {
        cpp!(unsafe [r as "quint16", g as "quint16", b as "quint16", a as "quint16"] -> QRgba64 as "QRgba64" {
            return QRgba64::fromRgba64(r, g, b, a);
        })
    }
}

// The representation always has the order red green blue alpha, so the format of c (the input value color)
// can be confusing since it has the same in-memory representation regardless of endian ordering but how one uses
// the interface changes

// impl From<u64> for QRgba64 {
//     fn from(val: u64) -> QRgba64 {
//         cpp!(unsafe [val as "quint64"] -> QRgba64 as "QRgba64" {
//             return QRgba64::fromRgba64(val);
//         })
//     }
// }

// #[cfg(test)]
// impl Into<u64> for QRgba64 {
//     /// Converts QRgba to a ABGR tuple
//     fn into(self) -> u64 {
//         cpp!(unsafe [self as "QRgba64"] -> u64 as "quint64" {
//             return (quint64)(self);
//         })
//     }
// }

impl From<QRgb> for QRgba64 {
    fn from(val: QRgb) -> QRgba64 {
        cpp!(unsafe [val as "unsigned int"] -> QRgba64 as "QRgba64" {
            return QRgba64::fromArgb32(val);
        })
    }
}

impl Into<QRgb> for QRgba64 {
    fn into(self) -> QRgb {
        let hex: u32 = self.into();
        hex.into()
    }
}

impl From<u32> for QRgba64 {
    fn from(val: u32) -> QRgba64 {
        cpp!(unsafe [val as "unsigned int"] -> QRgba64 as "QRgba64" {
            return QRgba64::fromArgb32(val);
        })
    }
}

impl Into<u32> for QRgba64 {
    fn into(self) -> u32 {
        cpp!(unsafe [self as "QRgba64"] -> u32 as "unsigned int" {
            return self.toArgb32();
        })
    }
}

impl Into<u16> for QRgba64 {
    fn into(self) -> u16 {
        cpp!(unsafe [self as "QRgba64"] -> u16 as "unsigned short" {
            return self.toRgb16();
        })
    }
}

impl std::fmt::Debug for QRgba64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QRgba64")
            .field("alpha", &self.alpha())
            .field("red", &self.red())
            .field("green", &self.green())
            .field("blue", &self.blue())
            .finish()
    }
}

/// Bindings for [`QColor::NameFormat`][class] enum class.
///
/// [class]: https://doc.qt.io/qt-5/qcolor.html#NameFormat-enum
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum QColorNameFormat {
    /// #RRGGBB A "#" character followed by three two-digit hexadecimal numbers (i.e. #RRGGBB).
    HexRgb = 0,
    ///#AARRGGBB A "#" character followed by four two-digit hexadecimal numbers (i.e. #AARRGGBB).
    HexArgb = 1,
}

/// Bindings for [`QColor::Spec`][class] enum class.
///
/// [class]: https://doc.qt.io/qt-5/qcolor.html#Spec-enum
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum QColorSpec {
    Invalid = 0,
    Rgb = 1,
    Hsv = 2,
    Cmyk = 3,
    Hsl = 4,
    ExtendedRgb = 5,
}

cpp_class!(
    /// Wrapper around [`QColor`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qcolor.html
    #[derive(Default, Clone, Copy, PartialEq)]
    pub unsafe struct QColor as "QColor"
);

impl QColor {
    /// Wrapper around [`QColor(QLatin1String)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qcolor.html#QColor-8
    pub fn from_name(name: &str) -> Self {
        let len = name.len();
        let ptr = name.as_ptr();
        cpp!(unsafe [len as "size_t", ptr as "char*"] -> QColor as "QColor" {
            return QColor(QLatin1String(ptr, len));
        })
    }

    /*
     * ==============
     * STATIC MEMBERS
     * ==============
     */

    // fn colorNames() -> QStringList;

    pub fn from_cmyk(c: i32, m: i32, y: i32, k: i32) -> QColor {
        Self::from_cmyka(c, m, y, k, 255)
    }

    pub fn from_cmyka(c: i32, m: i32, y: i32, k: i32, a: i32) -> QColor {
        cpp!(unsafe [c as "int", m as "int", y as "int", k as "int", a as "int"] -> QColor as "QColor" {
            return QColor::fromCmyk(c, m, y, k, a);
        })
    }

    pub fn from_cmyk_f(c: qreal, m: qreal, y: qreal, k: qreal) -> QColor {
        Self::from_cmyka_f(c, m, y, k, 1.0)
    }

    pub fn from_cmyka_f(c: qreal, m: qreal, y: qreal, k: qreal, a: qreal) -> QColor {
        cpp!(unsafe [c as "qreal", m as "qreal", y as "qreal", k as "qreal", a as "qreal"] -> QColor as "QColor" {
            return QColor::fromCmykF(c, m, y, k, a);
        })
    }

    pub fn from_hsl(h: i32, s: i32, l: i32) -> QColor {
        Self::from_hsla(h, s, l, 255)
    }

    pub fn from_hsla(h: i32, s: i32, l: i32, a: i32) -> QColor {
        cpp!(unsafe [h as "int", s as "int", l as "int", a as "int"] -> QColor as "QColor" {
            return QColor::fromHsl(h, s, l, a);
        })
    }

    pub fn from_hsl_f(h: qreal, s: qreal, l: qreal) -> QColor {
        Self::from_hsla_f(h, s, l, 1.0)
    }

    pub fn from_hsla_f(h: qreal, s: qreal, l: qreal, a: qreal) -> QColor {
        cpp!(unsafe [h as "qreal", s as "qreal", l as "qreal", a as "qreal"] -> QColor as "QColor" {
            return QColor::fromHslF(h, s, l, a);
        })
    }

    pub fn from_hsv(h: i32, s: i32, v: i32) -> QColor {
        Self::from_hsva(h, s, v, 255)
    }

    pub fn from_hsva(h: i32, s: i32, v: i32, a: i32) -> QColor {
        cpp!(unsafe [h as "int", s as "int", v as "int", a as "int"] -> QColor as "QColor" {
            return QColor::fromHsv(h, s, v, a);
        })
    }

    pub fn from_hsv_f(h: qreal, s: qreal, v: qreal) -> QColor {
        Self::from_hsva_f(h, s, v, 1.0)
    }

    pub fn from_hsva_f(h: qreal, s: qreal, v: qreal, a: qreal) -> QColor {
        cpp!(unsafe [h as "qreal", s as "qreal", v as "qreal", a as "qreal"] -> QColor as "QColor" {
            return QColor::fromHsvF(h, s, v, a);
        })
    }

    pub fn from_rgb(r: i32, g: i32, b: i32) -> QColor {
        Self::from_rgba(r, g, b, 255)
    }

    pub fn from_rgba(r: i32, g: i32, b: i32, a: i32) -> QColor {
        cpp!(unsafe [r as "int", g as "int", b as "int", a as "int"] -> QColor as "QColor" {
            return QColor::fromRgb(r, g, b, a);
        })
    }

    /// Wrapper around [`fromRgbF(qreal r, qreal g, qreal b, qreal a = 1.0)`][ctor] constructor.
    ///
    /// # Wrapper-specific
    ///
    /// Alpha is left at default `1.0`. To set it to something other that 1.0, use [`from_rgba_f`][].
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qcolor.html#fromRgbF
    /// [`from_rgba_f`]: #method.from_rgba_f
    pub fn from_rgb_f(r: qreal, g: qreal, b: qreal) -> QColor {
        cpp!(unsafe [r as "qreal", g as "qreal", b as "qreal"] -> QColor as "QColor" {
            return QColor::fromRgbF(r, g, b);
        })
    }

    /// Wrapper around [`fromRgbF(qreal r, qreal g, qreal b, qreal a = 1.0)`][ctor] constructor.
    ///
    /// # Wrapper-specific
    ///
    /// Same as [`from_rgb_f`][], but accept an alpha value
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qcolor.html#fromRgbF
    /// [`from_rgb_f`]: #method.from_rgb_f
    pub fn from_rgba_f(r: qreal, g: qreal, b: qreal, a: qreal) -> Self {
        cpp!(unsafe [r as "qreal", g as "qreal", b as "qreal", a as "qreal"] -> QColor as "QColor" {
            return QColor::fromRgbF(r, g, b, a);
        })
    }

    pub fn from_rgb64(r: u16, g: u16, b: u16) -> QColor {
        Self::from_rgba64(r, g, b, u16::MAX)
    }

    pub fn from_rgba64(r: u16, g: u16, b: u16, a: u16) -> QColor {
        cpp!(unsafe [r as "unsigned short", g as "unsigned short", b as "unsigned short", a as "unsigned short"] -> QColor as "QColor" {
            return QColor::fromRgba64(r, g, b, a);
        })
    }

    pub fn from_qrgba64(rgba64: QRgba64) -> QColor {
        cpp!(unsafe [rgba64 as "QRgba64"] -> QColor as "QColor" {
            return QColor::fromRgba64(rgba64);
        })
    }

    pub fn from_qrgb(rgb: QRgb) -> QColor {
        let rgb: u32 = rgb.into();
        cpp!(unsafe [rgb as "QRgb"] -> QColor as "QColor" {
            return QColor::fromRgb(rgb);
        })
    }

    pub fn is_valid_color(name: &str) -> bool {
        let len = name.len();
        let ptr = name.as_ptr();

        cpp!(unsafe [len as "size_t", ptr as "char*"] -> bool as "bool" {
            return QColor::isValidColor(QLatin1String(ptr, len));
        })
    }

    /*
     * ==============
     * Public MEMBERS
     * ==============
     */
    pub fn alpha(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->alpha();
        })
    }

    pub fn alpha_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->alphaF();
        })
    }

    pub fn black(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->black();
        })
    }

    pub fn black_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->blackF();
        })
    }

    pub fn blue(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->blue();
        })
    }

    pub fn blue_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->blueF();
        })
    }

    pub fn convert_to(&self, color_spec: QColorSpec) -> QColor {
        cpp!(unsafe [self as "const QColor*", color_spec as "QColor::Spec"] -> QColor as "QColor" {
            return self->convertTo(color_spec);
        })
    }

    pub fn cyan(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->cyan();
        })
    }

    pub fn cyan_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->cyanF();
        })
    }

    pub fn darker(&self, factor: Option<i32>) -> QColor {
        let factor = match factor {
            Some(factor) => factor,
            None => 200,
        };

        cpp!(unsafe [self as "const QColor*", factor as "int"] -> QColor as "QColor" {
            return self->darker(factor);
        })
    }

    /// This function should be const but at least in my local include (5.12) it is not marked as const and causes compilation to fail
    /// > void getCmyk(int *c, int *m, int *y, int *k, int *a = nullptr);
    pub fn get_cmyka(&mut self) -> (i32, i32, i32, i32, i32) {
        let res = (0, 0, 0, 0, 0);
        let (ref c, ref m, ref y, ref k, ref a) = res;
        cpp!(unsafe [self as "QColor*", c as "int*", m as "int*", y as "int*", k as "int*", a as "int*"] {
        #if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
            int c_, m_, y_, k_, a_;
            self->getCmyk(&c_, &m_, &y_, &k_, &a_);
            *c = c_; *m = m_; *y = y_; *k = k_; *a = a_;
        #else
            self->getCmyk(c, m, y, k, a);
        #endif
        });
        res
    }

    /// This function should be const but at least in my local include (5.12) it is not marked as const and causes compilation to fail
    /// > void getCmykF(qreal *c, qreal *m, qreal *y, qreal *k, qreal *a = nullptr);
    pub fn get_cmyka_f(&mut self) -> (qreal, qreal, qreal, qreal, qreal) {
        let res = (0., 0., 0., 0., 0.);
        let (ref c, ref m, ref y, ref k, ref a) = res;
        cpp!(unsafe [self as "QColor*", c as "qreal*", m as "qreal*", y as "qreal*", k as "qreal*", a as "qreal*"] {
        #if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
            float c_, m_, y_, k_, a_;
            self->getCmykF(&c_, &m_, &y_, &k_, &a_);
            *c = c_; *m = m_; *y = y_; *k = k_; *a = a_;
        #else
            self->getCmykF(c, m, y, k, a);
        #endif
        });
        res
    }

    pub fn get_hsla(&self) -> (i32, i32, i32, i32) {
        let res = (0, 0, 0, 0);
        let (ref h, ref s, ref l, ref a) = res;
        cpp!(unsafe [self as "const QColor*", h as "int*", s as "int*", l as "int*", a as "int*"] {
        #if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
            int h_, s_, l_, a_;
            self->getHsl(&h_, &s_, &l_, &a_);
            *h = h_; *s = s_; *l = l_; *a = a_;
        #else
            self->getHsl(h, s, l, a);
        #endif
        });
        res
    }

    pub fn get_hsla_f(&self) -> (qreal, qreal, qreal, qreal) {
        let res = (0., 0., 0., 0.);
        let (ref h, ref s, ref l, ref a) = res;
        cpp!(unsafe [self as "const QColor*", h as "qreal*", s as "qreal*", l as "qreal*", a as "qreal*"] {
        #if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
            float h_, s_, l_, a_;
            self->getHslF(&h_, &s_, &l_, &a_);
            *h = h_; *s = s_; *l = l_; *a = a_;
        #else
            return self->getHslF(h, s, l, a);
        #endif
        });
        res
    }

    pub fn get_hsva(&self) -> (i32, i32, i32, i32) {
        let res = (0, 0, 0, 0);
        let (ref h, ref s, ref v, ref a) = res;
        cpp!(unsafe [self as "const QColor*", h as "int*", s as "int*", v as "int*", a as "int*"] {
        #if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
            int h_, s_, v_, a_;
            self->getHsv(&h_, &s_, &v_, &a_);
            *h = h_; *s = s_; *v = v_; *a = a_;
        #else
            self->getHsv(h, s, v, a);
        #endif
        });
        res
    }

    pub fn get_hsva_f(&self) -> (qreal, qreal, qreal, qreal) {
        let res = (0., 0., 0., 0.);
        let (ref h, ref s, ref v, ref a) = res;
        cpp!(unsafe [self as "const QColor*", h as "qreal*", s as "qreal*", v as "qreal*", a as "qreal*"] {
        #if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
            float h_, s_, v_, a_;
            self->getHsvF(&h_, &s_, &v_, &a_);
            *h = h_; *s = s_; *v = v_; *a = a_;
        #else
            return self->getHsvF(h, s, v, a);
        #endif
        });
        res
    }

    /// Wrapper around [`getRgbF(qreal *r, qreal *g, qreal *b, qreal *a = nullptr)`][method] method.
    ///
    /// # Wrapper-specific
    ///
    /// Returns red, green, blue and alpha components as a tuple, instead of mutable references.
    ///
    /// [method]: https://doc.qt.io/qt-5/qcolor.html#getRgbF
    pub fn get_rgba(&self) -> (i32, i32, i32, i32) {
        let res = (0, 0, 0, 0);
        let (ref r, ref g, ref b, ref a) = res;
        cpp!(unsafe [self as "const QColor*", r as "int*", g as "int*", b as "int*", a as "int*"] {
        #if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
            int r_, g_, b_, a_;
            self->getRgb(&r_, &g_, &b_, &a_);
            *r = r_; *g = g_; *b = b_; *a = a_;
        #else
            return self->getRgb(r, g, b, a);
        #endif
        });
        res
    }

    pub fn get_rgba_f(&self) -> (qreal, qreal, qreal, qreal) {
        let res = (0., 0., 0., 0.);
        let (ref r, ref g, ref b, ref a) = res;
        cpp!(unsafe [self as "const QColor*", r as "qreal*", g as "qreal*", b as "qreal*", a as "qreal*"] {
        #if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
            float r_, g_, b_, a_;
            self->getRgbF(&r_, &g_, &b_, &a_);
            *r = r_; *g = g_; *b = b_; *a = a_;
        #else
            return self->getRgbF(r, g, b, a);
        #endif
        });
        res
    }

    pub fn green(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->green();
        })
    }

    pub fn green_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->greenF();
        })
    }

    pub fn hsl_hue(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->hslHue();
        })
    }
    pub fn hsl_hue_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->hslHueF();
        })
    }
    pub fn hsl_saturation(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->hslSaturation();
        })
    }
    pub fn hsl_saturation_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->hslSaturationF();
        })
    }
    pub fn hsv_hue(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->hsvHue();
        })
    }
    pub fn hsv_hue_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->hsvHueF();
        })
    }

    pub fn hsv_saturation(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->hsvSaturation();
        })
    }

    pub fn hsv_saturation_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->hsvSaturationF();
        })
    }

    pub fn is_valid(&self) -> bool {
        cpp!(unsafe [self as "const QColor*"] -> bool as "bool" {
            return self->isValid();
        })
    }

    pub fn lighter(&self, factor: Option<i32>) -> QColor {
        let factor = match factor {
            Some(factor) => factor,
            None => 150,
        };

        cpp!(unsafe [self as "const QColor*", factor as "int"] -> QColor as "QColor" {
            return self->lighter(factor);
        })
    }

    pub fn lightness(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->lightness();
        })
    }

    pub fn lightness_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->lightnessF();
        })
    }

    pub fn magenta(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->magenta();
        })
    }

    pub fn magenta_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->magentaF();
        })
    }

    pub fn name(&self) -> QString {
        cpp!(unsafe [self as "const QColor*"] -> QString as "QString" {
            return self->name();
        })
    }

    pub fn name_with_format(&self, format: QColorNameFormat) -> QString {
        cpp!(unsafe [self as "const QColor*", format as "QColor::NameFormat"] -> QString as "QString" {
            return self->name(format);
        })
    }

    pub fn red(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->red();
        })
    }

    pub fn red_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->redF();
        })
    }

    pub fn rgb(&self) -> QRgb {
        QRgb::from(cpp!(unsafe [self as "const QColor*"] -> u32 as "QRgb" {
            return self->rgb();
        }))
    }

    pub fn rgba64(&self) -> QRgba64 {
        cpp!(unsafe [self as "const QColor*"] -> QRgba64 as "QRgba64" {
            return QRgba64::fromRgba64(self->rgba64());
        })
    }

    pub fn rgba(&self) -> QRgb {
        QRgb::from(cpp!(unsafe [self as "const QColor*"] -> u32 as "QRgb" {
            return self->rgba();
        }))
    }

    pub fn set_alpha(&mut self, alpha: i32) {
        cpp!(unsafe [self as "QColor*", alpha as "int"] {
            return self->setAlpha(alpha);
        })
    }

    pub fn set_alpha_f(&mut self, alpha: qreal) {
        cpp!(unsafe [self as "QColor*", alpha as "qreal"] {
            return self->setAlphaF(alpha);
        })
    }

    pub fn set_blue(&mut self, blue: i32) {
        cpp!(unsafe [self as "QColor*", blue as "int"] {
            return self->setBlue(blue);
        })
    }

    pub fn set_blue_f(&mut self, blue: qreal) {
        cpp!(unsafe [self as "QColor*", blue as "qreal"] {
            return self->setBlueF(blue);
        })
    }

    pub fn set_cmyk(&mut self, c: i32, m: i32, y: i32, k: i32, a: Option<i32>) {
        let a = match a {
            Some(a) => a,
            None => 255,
        };
        cpp!(unsafe [self as "QColor*", c as "int", m as "int", y as "int", k as "int", a as "int"] {
            return self->setCmyk(c, m, y, k, a);
        })
    }

    pub fn set_cmyk_f(&mut self, c: qreal, m: qreal, y: qreal, k: qreal, a: Option<qreal>) {
        let a = match a {
            Some(a) => a,
            None => 1.0,
        };
        cpp!(unsafe [self as "QColor*", c as "qreal", m as "qreal", y as "qreal", k as "qreal", a as "qreal"] {
            return self->setCmykF(c, m, y, k, a);
        })
    }

    pub fn set_green(&mut self, green: i32) {
        cpp!(unsafe [self as "QColor*", green as "int"] {
            return self->setGreen(green);
        })
    }

    pub fn set_green_f(&mut self, green: qreal) {
        cpp!(unsafe [self as "QColor*", green as "qreal"] {
            return self->setGreenF(green);
        })
    }

    pub fn set_hsl(&mut self, h: i32, s: i32, l: i32, a: Option<i32>) {
        let a = match a {
            Some(a) => a,
            None => 255,
        };
        cpp!(unsafe [self as "QColor*", h as "int", s as "int", l as "int", a as "int"] {
            return self->setHsl(h, s, l, a);
        })
    }

    pub fn set_hsl_f(&mut self, h: qreal, s: qreal, l: qreal, a: Option<qreal>) {
        let a = match a {
            Some(a) => a,
            None => 1.0,
        };
        cpp!(unsafe [self as "QColor*", h as "qreal", s as "qreal", l as "qreal", a as "qreal"] {
            return self->setHslF(h, s, l, a);
        })
    }

    pub fn set_hsv(&mut self, h: i32, s: i32, v: i32, a: Option<i32>) {
        let a = match a {
            Some(a) => a,
            None => 255,
        };
        cpp!(unsafe [self as "QColor*", h as "int", s as "int", v as "int", a as "int"] {
            return self->setHsv(h, s, v, a);
        })
    }

    pub fn set_hsv_f(&mut self, h: qreal, s: qreal, v: qreal, a: Option<qreal>) {
        let a = match a {
            Some(a) => a,
            None => 1.0,
        };
        cpp!(unsafe [self as "QColor*", h as "qreal", s as "qreal", v as "qreal", a as "qreal"] {
            return self->setHsvF(h, s, v, a);
        })
    }

    pub fn set_named_color(&mut self, name: &str) {
        let len = name.len();
        let ptr = name.as_ptr();
        cpp!(unsafe [self as "QColor*", len as "size_t", ptr as "char*"] {
            return self->setNamedColor(QLatin1String(ptr, len));
        })
    }

    pub fn set_red(&mut self, red: i32) {
        cpp!(unsafe [self as "QColor*", red as "int"] {
            return self->setRed(red);
        })
    }

    pub fn set_red_f(&mut self, red: qreal) {
        cpp!(unsafe [self as "QColor*", red as "qreal"] {
            return self->setRedF(red);
        })
    }

    pub fn set_rgb(&mut self, r: i32, g: i32, b: i32, a: Option<i32>) {
        let a = match a {
            Some(a) => a,
            None => 255,
        };
        cpp!(unsafe [self as "QColor*", r as "int", g as "int", b as "int", a as "int"] {
            return self->setRgb(r, g, b, a);
        })
    }

    pub fn set_qrgb(&mut self, rgb: QRgb) {
        let rgb: u32 = rgb.0;
        cpp!(unsafe [self as "QColor*", rgb as "QRgb"] {
            return self->setRgb(rgb);
        })
    }

    pub fn set_rgba_64(&mut self, rgba: QRgba64) {
        cpp!(unsafe [self as "QColor*", rgba as "QRgba64"] {
            return self->setRgba64(rgba);
        })
    }

    pub fn set_rgb_f(&mut self, r: qreal, g: qreal, b: qreal, a: Option<qreal>) {
        let a = match a {
            Some(a) => a,
            None => 1.0,
        };
        cpp!(unsafe [self as "QColor*", r as "qreal", g as "qreal", b as "qreal", a as "qreal"] {
            return self->setRgbF(r, g, b, a);
        })
    }

    pub fn set_rgba(&mut self, rgba: QRgb) {
        let rgba: u32 = rgba.into();
        cpp!(unsafe [self as "QColor*", rgba as "QRgb"] {
            return self->setRgba(rgba);
        })
    }

    pub fn spec(&self) -> QColorSpec {
        cpp!(unsafe [self as "const QColor*"] -> QColorSpec as "QColor::Spec" { return self->spec(); })
    }

    pub fn to_cmyk(&self) -> QColor {
        cpp!(unsafe [self as "const QColor*"] -> QColor as "QColor" {
            return self->toCmyk();
        })
    }

    // #[cfg(qt_5_14)]
    // fn toExtendedRgb(&self) -> QColor {
    //     cpp!(unsafe [self as "const QColor*"] -> QColor as "QColor" {
    //         return self->toExtendedRgb();
    //     })
    // }

    pub fn to_hsl(&self) -> QColor {
        cpp!(unsafe [self as "const QColor*"] -> QColor as "QColor" {
            return self->toHsl();
        })
    }

    pub fn to_hsv(&self) -> QColor {
        cpp!(unsafe [self as "const QColor*"] -> QColor as "QColor" {
            return self->toHsv();
        })
    }

    pub fn to_rgb(&self) -> QColor {
        cpp!(unsafe [self as "const QColor*"] -> QColor as "QColor" {
            return self->toRgb();
        })
    }

    pub fn value(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->value();
        })
    }

    pub fn value_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->valueF();
        })
    }

    pub fn yellow(&self) -> i32 {
        cpp!(unsafe [self as "const QColor*"] -> i32 as "int" {
            return self->yellow();
        })
    }

    pub fn yellow_f(&self) -> qreal {
        cpp!(unsafe [self as "const QColor*"] -> qreal as "qreal" {
            return self->yellowF();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qcolor_from_name() {
        let blue1 = QColor::from_name("blue");
        let blue2 = QColor::from_rgb_f(0., 0., 1.);
        assert_eq!(blue1.get_rgba_f().0, 0.);
        assert_eq!(blue1.get_rgba_f().2, 1.);
        assert!(blue1 == blue2);

        let red1 = QColor::from_name("red");
        let red2 = QColor::from_rgb_f(1., 0., 0.);
        assert_eq!(red1.get_rgba_f().0, 1.);
        assert_eq!(red1.get_rgba_f().2, 0.);
        assert!(red1 == red2);
        assert!(blue1 != red1);
    }

    #[test]
    fn test_rgb() {
        let color = QColor::from_rgb(255, 128, 0);
        assert_eq!(255, color.red());
        assert_eq!(128, color.green());
        assert_eq!(0, color.blue());
        assert_eq!(255, color.alpha());
        assert_eq!((255, 128, 0, 255), color.get_rgba());
    }

    #[test]
    fn test_cmyk() {
        let mut color = QColor::from_cmyk(255, 200, 100, 0);
        assert_eq!(255, color.cyan());
        assert_eq!(200, color.magenta());
        assert_eq!(100, color.yellow());
        assert_eq!(0, color.black());
        assert_eq!(255, color.alpha());
        assert_eq!((255, 200, 100, 0, 255), color.get_cmyka());
    }

    #[test]
    fn test_hsl() {
        let color = QColor::from_hsla(255, 200, 100, 213);
        assert_eq!(255, color.hsl_hue());
        assert_eq!(200, color.hsl_saturation());
        assert_eq!(100, color.lightness());
        assert_eq!(213, color.alpha());
        assert_eq!((255, 200, 100, 213), color.get_hsla());
    }

    #[test]
    fn test_hsv() {
        let color = QColor::from_hsva(255, 200, 100, 213);
        assert_eq!(255, color.hsv_hue());
        assert_eq!(200, color.hsv_saturation());
        assert_eq!(100, color.value());
        assert_eq!(213, color.alpha());
        assert_eq!((255, 200, 100, 213), color.get_hsva());
    }

    #[test]
    fn test_three_way_equivalence() {
        // Test three-way equivalence
        let hex = 0xffeeddcc;
        let rgb: QRgb = hex.into();
        assert_eq!(hex, rgb.into());

        let rgb64: QRgba64 = hex.into();
        assert_eq!(hex, rgb64.into());

        assert_eq!(rgb, rgb64.into());
        assert_eq!(rgb64, rgb.into());
    }

    #[test]
    fn test_grayscale() {
        let rgb: QRgb = QRgb::rgba(0xee, 0xdd, 0xcc, 0xff);
        let a = rgb.alpha();
        let r = rgb.red();
        let g = rgb.green();
        let b = rgb.blue();

        assert_eq!((0xff, 0xee, 0xdd, 0xcc), (a, r, g, b));
        let gray: i32 = ((r as i32) * 11 + (g as i32) * 16 + (b as i32) * 5) / 32;
        assert_eq!(gray as u8, rgb.grayscale());
    }

    #[test]
    fn test_qrgba64_getters() {
        let rgb = QRgba64::from_rgba(0xffff, 0xeeee, 0xdddd, 0xcccc);
        let a = rgb.alpha();
        let r = rgb.red();
        let g = rgb.green();
        let b = rgb.blue();

        assert_eq!((0xffff, 0xeeee, 0xdddd, 0xcccc), (r, g, b, a));
    }

    #[test]
    fn test_conversions() {
        let rgb = QRgba64::from_rgba(0xffff, 0xeeee, 0xdddd, 0xcccc);
        let r = rgb.red();
        let g = rgb.green();
        let b = rgb.blue();

        // u32 conversion
        let hex32 = rgb.into();
        assert_eq!(0xccffeedd as u32, hex32);

        // u16 conversion (this is the method Qt uses to calculate it)
        let rgb16 = (r & 0xf800) | ((g >> 10) << 5) | (b >> 11);
        assert_eq!(rgb16, rgb.into());
    }
}
