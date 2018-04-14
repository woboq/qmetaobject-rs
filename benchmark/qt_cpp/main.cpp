#include <QtQuick/QtQuick>

class MyCppClass : public QObject {
    Q_OBJECT
public:
    Q_INVOKABLE int addTwo(int x) { return x+2; }
    Q_INVOKABLE int countW(const QString &x) {
        int count = 0;
        for(int i = 0; i < x.size(); ++i) {
            if(x[i] == 'W')
                count++;
        }
        return count;
    }
    Q_INVOKABLE QString replaceW(const QString &x) {
        auto y = x;
        return y.replace("W", ".");
    }

    QString strProp = "Hello";
    Q_PROPERTY(QString strProp MEMBER strProp)
    int intProp = 42;
    Q_PROPERTY(int intProp MEMBER intProp)
};

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine(argv[1]);
    auto robjs = engine.rootObjects();
    if (robjs.isEmpty())
        return false;
    QVariant b;
    qDebug() << "FROM JS";
    if (!QMetaObject::invokeMethod(robjs.first(), "benchmark", Q_ARG(QVariant,QVariant::fromValue<QObject*>(robjs.first()))))
        qWarning() << "calling 'doTest' failed";
    qDebug() << "FROM CPP";
    MyCppClass obj;
    if (!QMetaObject::invokeMethod(robjs.first(), "benchmark", Q_ARG(QVariant,QVariant::fromValue<QObject*>(&obj))))
        qWarning() << "calling 'doTest' failed";
    return 0;
}

#include "main.moc"
