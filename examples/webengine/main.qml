import QtQuick 2.6;
import QtQuick.Window 2.0;
import QtWebEngine 1.4

Window {
    visible: true
    title: "WebEngineView"
    width: 800
    height: 600
    WebEngineView {
        anchors.fill: parent
        url: "qrc:/webengine/index.html"
    }
}