/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
#pragma once

#include <QtCore/QObject>
#include <QtCore/QEvent>
#include <QtCore/QDebug>

union SignalCppRepresentation {
    void (QObject::*cpp_signal)();
    qintptr rust_signal;

    SignalCppRepresentation() = default;

    // Construct the object from an arbirary signal.
    // (there is a double indirection in the reinterpret_cast to avoid -Wcast-function-type)
    template<typename R, typename ...Args>
    SignalCppRepresentation(R (QObject::*cpp_signal)(Args...))
        : cpp_signal(*reinterpret_cast<void (QObject::**)()>(&cpp_signal)) { }
};

struct TraitObject {
    void *a;
    void *b;
    explicit operator bool() const { return a && b; }
};

extern "C" QMetaObject *RustObject_metaObject(TraitObject);
extern "C" void RustObject_destruct(TraitObject);

template <typename Base>
struct RustObject : Base {
    TraitObject rust_object;
    void (*extra_destruct)(QObject *);
    const QMetaObject *metaObject() const override {
        return rust_object ? RustObject_metaObject(rust_object) : Base::metaObject();
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
        if (rust_object && event->type() == 513) {
            // "513 reserved for Qt Jambi's DeleteOnMainThread event"
            // This event is sent by rust when we are deleted.
            rust_object = { nullptr, nullptr }; // so the destructor don't recurse
            delete this;
            return true;
        }
        return Base::event(event);
    }
    ~RustObject() {
        auto r = rust_object;
        rust_object = { nullptr, nullptr };
        if (extra_destruct)
            extra_destruct(this);
        if (r)
            RustObject_destruct(r);
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


namespace QtPrivate {
// Hack to access QMetaType::registerConverterFunction which is private, but ConverterFunctor
// is a friend
template<>
struct ConverterFunctor<TraitObject, TraitObject, TraitObject> : public AbstractConverterFunction
{
    using AbstractConverterFunction::AbstractConverterFunction;
    bool registerConverter(int from, int to) {
        return QMetaType::registerConverterFunction(this, from, to);
    }
};
}
