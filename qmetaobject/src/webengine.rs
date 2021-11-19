use cpp::cpp;

cpp! {{
#if !(_WIN32 && ! defined(_MSC_VER))
#  if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
#    if QT_VERSION >= QT_VERSION_CHECK(6, 2, 0)
#      include <QtWebEngineQuick/QtWebEngineQuick>
#    endif
#  else
#    include <QtWebEngine/QtWebEngine>
#  endif
#endif
}}

/// Refer to the Qt documentation of QtWebEngine::initialize()
pub fn initialize() {
    cpp!(unsafe [] {
    #if !(_WIN32 && ! defined(_MSC_VER))
    #  if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
    #    if QT_VERSION >= QT_VERSION_CHECK(6, 2, 0)
        QtWebEngineQuick::initialize();
    #    endif
    #  else
        QtWebEngine::initialize();
    #  endif
    #endif
        });
}
