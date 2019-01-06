use qttypes::QString;

/// Refer to the Qt documentation for QQuickStyle
pub struct QQuickStyle {}

impl QQuickStyle {
    /// Refer to the Qt documentation for QQuickStyle::setStyle
    pub fn set_style(style: QString) {
        unsafe {
            cpp! {{
                #include <QtQuickControls2/QQuickStyle>
            }}

            cpp! {[style as "QString"] {
                 QQuickStyle::setStyle(style);
            }}
        }
    }
}
