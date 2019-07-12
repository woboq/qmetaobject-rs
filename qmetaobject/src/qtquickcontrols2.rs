/// Refer to the Qt documentation for QQuickStyle
pub struct QQuickStyle {}

impl QQuickStyle {
    /// Refer to the Qt documentation for QQuickStyle::setStyle
    pub fn set_style(style: &str) {
        std::env::set_var("QT_QUICK_CONTROLS_STYLE", style);
    }
}
