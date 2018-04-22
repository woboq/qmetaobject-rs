#pragma once

#include <QtCore/QObject>
#include <QtCore/QEvent>
#include <QtCore/QDebug>

struct TraitObject {
    void *a;
    void *b;
};

extern "C" QMetaObject *RustObject_metaObject(TraitObject);
extern "C" void RustObject_destruct(TraitObject);

template <typename Base>
struct RustObject : Base {
    TraitObject rust_object;
    void (*extra_destruct)(QObject *);
    const QMetaObject *metaObject() const override {
        return RustObject_metaObject(rust_object);
    }
    int qt_metacall(QMetaObject::Call _c, int _id, void **_a) override {
        _id = Base::qt_metacall(_c, _id, _a);
        if (_id < 0)
            return _id;
        const QMetaObject *mo = metaObject();
        if (_c == QMetaObject::InvokeMetaMethod || _c == QMetaObject::RegisterMethodArgumentMetaType) {
            int methodCount = mo->methodCount();
            if (_id < methodCount)
                mo->d.static_metacall(this, _c, _id, _a);
            _id -= methodCount;
        } else if ((_c >= QMetaObject::ReadProperty && _c <= QMetaObject::QueryPropertyUser)
            || _c == QMetaObject::RegisterPropertyMetaType) {
            int propertyCount = mo->propertyCount();
            if (_id < propertyCount)
                mo->d.static_metacall(this, _c, _id, _a);
            _id -= propertyCount;
        }
        return _id;
    }
    bool event(QEvent *event) override {
        if (event->type() == 513) {
            // "513 reserved for Qt Jambi's DeleteOnMainThread event"
            // This event is sent by rust when we are deleted.
            rust_object = { nullptr, nullptr }; // so the destructor don't recurse
            delete this;
            return true;
        }
        return Base::event(event);
    }
    ~RustObject() {
        if (extra_destruct)
            extra_destruct(this);
        if (rust_object.a || rust_object.b)
            RustObject_destruct(rust_object);
    }
};

struct RustObjectDescription {
    size_t size;
    const QMetaObject *baseMetaObject;
    QObject *(*create)(const TraitObject*);
    void (*qmlConstruct)(void*, const TraitObject*, void (*extra_destruct)(QObject *));
};

template<typename T>
const RustObjectDescription *rustObjectDescription() {
    static RustObjectDescription desc {
        sizeof(T),
        &T::staticMetaObject,
        [](const TraitObject *self) -> QObject* {
            auto q = new T();
            q->rust_object = *self;
            return q;
        },
        [](void *data, const TraitObject *self, void (*extra_destruct)(QObject *)) {
            auto *q = new (data) T();
            q->rust_object = *self;
            q->extra_destruct = extra_destruct;
        },

    };
    return &desc;
}

using CreatorFunction = void (*)(void *);


// Hack to access QMetaType::registerConverterFunction which is private, but ConverterFunctor
// is a friend
namespace QtPrivate {
template<>
struct ConverterFunctor<TraitObject, TraitObject, TraitObject> : public AbstractConverterFunction
{
    using AbstractConverterFunction::AbstractConverterFunction;
    bool registerConverter(int from, int to) {
        return QMetaType::registerConverterFunction(this, from, to);
    }
};
}
