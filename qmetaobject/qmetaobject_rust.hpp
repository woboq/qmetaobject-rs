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

    // Construct the object from an arbitrary signal.
    // (there is a double indirection in the reinterpret_cast to avoid -Wcast-function-type)
    template<typename R, typename Object, typename ...Args>
    SignalCppRepresentation(R (Object::*cpp_signal)(Args...))
        : cpp_signal(*reinterpret_cast<void (QObject::**)()>(&cpp_signal)) { }
};

struct TraitObject {
    void *a;
    void *b;
    explicit operator bool() const { return a && b; }
};

extern "C" QMetaObject *RustObject_metaObject(TraitObject);
extern "C" void RustObject_destruct(TraitObject);

/// "513 reserved for Qt Jambi's DeleteOnMainThread event"
/// We are just re-using this event type for our purposes.
///
/// Source: https://github.com/qtjambi/qtjambi/blob/8ef99da63315945e6ab540cc31d66e5b021b69e4/src/cpp/qtjambi/qtjambidebugevent.cpp#L857
static constexpr int QtJambi_EventType_DeleteOnMainThread = 513;

template <typename Base>
struct RustObject : Base {
    TraitObject rust_object;  // A QObjectPinned<XXX> where XXX is the base trait
    TraitObject ptr_qobject;  // a QObjectPinned<QObject>
    void (*extra_destruct)(QObject *);
    const QMetaObject *metaObject() const override {
        return ptr_qobject ? RustObject_metaObject(ptr_qobject) : Base::metaObject();
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
        if (ptr_qobject && event->type() == QtJambi_EventType_DeleteOnMainThread) {
            // This event is sent by rust when we are deleted.
            ptr_qobject = { nullptr, nullptr }; // so the destructor don't recurse
            delete this;
            return true;
        }
        return Base::event(event);
    }
    ~RustObject() {
        auto r = ptr_qobject;
        ptr_qobject = { nullptr, nullptr };
        if (extra_destruct)
            extra_destruct(this);
        if (r)
            RustObject_destruct(r);
    }
};

struct RustObjectDescription {
    size_t size;
    const QMetaObject *baseMetaObject;
    QObject *(*create)(const TraitObject *, const TraitObject *);
    void (*qmlConstruct)(void *, const TraitObject *, const TraitObject *, void (*extra_destruct)(QObject *));
    TraitObject (*get_rust_refcell)(QObject *); // Possible optimisation: make this an offset
};

template<typename T>
const RustObjectDescription *rustObjectDescription() {
    static RustObjectDescription desc {
        sizeof(T),
        &T::staticMetaObject,
        [](const TraitObject *self_pinned, const TraitObject *self_ptr) -> QObject* {
            auto q = new T();
            q->ptr_qobject = *self_ptr;
            q->rust_object = *self_pinned;
            return q;
        },
        [](void *data, const TraitObject *self_pinned, const TraitObject *self_ptr,
                void (*extra_destruct)(QObject *)) {
            auto *q = new (data) T();
            q->rust_object = *self_pinned;
            q->ptr_qobject = *self_ptr;
            q->extra_destruct = extra_destruct;
        },
        [](QObject *q) { return static_cast<T *>(q)->ptr_qobject; }
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
