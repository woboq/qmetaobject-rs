use cpp::cpp;

cpp! {{
#include <QtWebEngine/QtWebEngine>
}}

/// Refer to the Qt documentation of QtWebEngine::initialize()
pub fn initialize() {
    cpp!(unsafe [] {
        QtWebEngine::initialize();
    });
}
