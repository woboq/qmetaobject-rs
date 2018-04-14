import QtQuick 2.10
import QtQuick.Window 2.10

Window {
    visible: true;
    width: 455;
    height: 455;
    MouseArea {
        anchors.fill: parent;
        onClicked: {
            benchmark(testObj);
        }
    }

    function addTwo(x) { return x+2 }
    function countW(x) {
        var count = 0;
        for(var i = 0; i < x.length; ++i) {
            if(x[i] == 'W')
                count++;
        }
        return count;
    }
    function replaceW(x) { return x.replace("W", ".") }

    property string strProp : "hello";
    property int intProp : 42;

    function benchmark(item) {
        function mesure(text, f) {
            var startTime = new Date().getTime();
            var x;
            for (var i = 0; i < 1000000; ++i)
                x=f();
            console.log("ELAPSED: " + text + " in " + (new Date().getTime() - startTime));
            return x;
        }
        mesure("addTwo",  function() {
            return item.addTwo(2) + item.addTwo(3) + item.addTwo(4);
        });
        mesure("countW",  function() {
            return item.countW("sss") +
            item.countW("sssssssssssssssWssssssssssssssWsssssssssssssssssssssssss");
        });
        mesure("replaceW",  function() {
            item.replaceW("sss");
            return item.replaceW("sssssssssssssssWssssssssssssssWsssssssssssssssssssssssss");
        });

        mesure("strProp",  function() {
            var old = item.strProp;
            item.strProp = "dsdsdsdsdsds";
            item.strProp = old;
            return old;
        });

        mesure("intProp",  function() {
            var old = item.intProp;
            item.intProp += 42;
            item.intProp = old;
            return old;
        });
    }
}
