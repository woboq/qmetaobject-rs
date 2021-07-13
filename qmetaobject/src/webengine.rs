use cpp::cpp;

cpp! {{
#if !(QT_VERSION >= QT_VERSION_CHECK(6, 0, 0) && QT_VERSION < QT_VERSION_CHECK(6, 2, 0))
#  if !(_WIN32 && ! defined(_MSC_VER))
    #include <QtWebEngine/QtWebEngine>
#  endif
#endif
}}

/// Refer to the Qt documentation of QtWebEngine::initialize()
pub fn initialize() {
    cpp!(unsafe [] {
    #if !(QT_VERSION >= QT_VERSION_CHECK(6, 0, 0) && QT_VERSION < QT_VERSION_CHECK(6, 2, 0))
    #  if !(_WIN32 && ! defined(_MSC_VER))
        QtWebEngine::initialize();
    #  endif
    #endif
    });
}
