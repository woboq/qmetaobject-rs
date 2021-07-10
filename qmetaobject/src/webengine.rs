use cpp::cpp;

cpp! {{
#if !(QT_VERSION >= QT_VERSION_CHECK(6, 0, 0) && QT_VERSION < QT_VERSION_CHECK(6, 2, 0))
    #include <QtWebEngine/QtWebEngine>
#endif
}}

/// Refer to the Qt documentation of QtWebEngine::initialize()
pub fn initialize() {
    cpp!(unsafe [] {
    #if !(QT_VERSION >= QT_VERSION_CHECK(6, 0, 0) && QT_VERSION < QT_VERSION_CHECK(6, 2, 0))
        QtWebEngine::initialize();
    #endif
    });
}
