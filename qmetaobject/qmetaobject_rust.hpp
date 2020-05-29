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

/// Raw representation of a pointer to member function as specified by Itanium ABI.
/// This is somehow similar to `std::raw::TraitObject` in Rust, but for C++.
///
/// Note: it can not be used to represent a pointer to data member.
///
/// http://itanium-cxx-abi.github.io/cxx-abi/abi.html#member-pointers
struct MemberFunctionPtr {
    ptrdiff_t ptr;
    ptrdiff_t adj;

    /// Create raw pointer to member function from raw parts.
    explicit MemberFunctionPtr(ptrdiff_t ptr, ptrdiff_t adj)
        : ptr(ptr)
        , adj(adj)
    {}

    /// Create raw pointer to member function from a reference to any member
    /// function of any type.
    template<typename R, typename Type, typename ...Args>
    explicit MemberFunctionPtr(R (Type::* func)(Args...)) {
        *this = *reinterpret_cast<MemberFunctionPtr *>(&func);
    }
};

/// Pointer to a method of QObject which takes no arguments and returns nothing.
/// Actually this is a "type-erased" method with various arguments and return
/// value, but it merely represents a generic pointer, and let other code
/// handle the types and memory safety.
using QObjectMethodErased = void (QObject::*)();

/// From `QMetaObject::Connection QObject::connectImpl(...)` documentation:
/// ```
/// signal is a pointer to a pointer to a member signal of the sender
/// ```
///
/// This type encapsulates a pointer to a member function and provides handy
/// conversions.
union SignalCppRepresentation {
// No need to be public. Pointer to a signal is exposed via safe public getter.
private:
    /// Tear fat pointer apart.
    MemberFunctionPtr raw;
    /// Or take it as an erased pointer to member function. Useless on its own,
    /// but provides better options for `reinterpret_cast`.
    QObjectMethodErased erased;

public:
    /// Construct from Rust side, when type information is not available to C++.
    /// Member function must be a QObject signal.
    explicit SignalCppRepresentation(MemberFunctionPtr ptr)
        : raw(ptr)
    {}

    /// Same as `SignalCppRepresentation(MemberFunctionPtr)`, but assumes no
    /// adjustment to `this`, i.e. `MemberFunctionPtr::adj = 0`.
    explicit SignalCppRepresentation(ptrdiff_t ptr)
        : SignalCppRepresentation(MemberFunctionPtr(ptr, 0))
    {}

    /// Construct the object from an arbitrary Qt signal.
    ///
    /// Note: this is an implicit conversion.
    template<typename R, typename Type, typename ...Args>
    SignalCppRepresentation(R (Type::* qt_signal)(Args...))
        // (there is a double indirection in the reinterpret_cast to avoid -Wcast-function-type)
        : erased(*reinterpret_cast<QObjectMethodErased *>(&qt_signal))
    {}

    /// Qt uses "pointer to a pointer to a member" signal representation inside
    /// QObject::connect(...) functions. This little helper encapsulates the
    /// required cast.
    void **asRawSignal() {
        return reinterpret_cast<void **>(&erased);
    }
};

/// Wrapper for Rust `std::raw::TraitObject` struct.
///
/// Note: `std::raw` is marked unstable as of Rust 1.43.0, so for future
/// compatibility it would be better to box the trait object on the heap,
/// and never manipulate its content directly from C++. For the time being,
/// though, let it be.
struct TraitObject {
    void *data;
    void *vtable;

    /// Nullability check.
    bool isValid() const {
        return data && vtable;
    }

    /// Forget about referenced object.
    ///
    /// If this TraitObject represented a `Box` (owned object) rather than a
    /// `&dyn` reference (borrowed object) then it may cause memory leaks,
    /// unless a copy was made for later proper destruction.
    inline void invalidate() {
        data = nullptr;
        vtable = nullptr;
    }
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
        return ptr_qobject.isValid() ? RustObject_metaObject(ptr_qobject) : Base::metaObject();
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
        if (ptr_qobject.isValid() && event->type() == QtJambi_EventType_DeleteOnMainThread) {
            // This event is sent by rust when we are deleted.
            ptr_qobject.invalidate(); // so the destructor don't recurse
            delete this;
            return true;
        }
        return Base::event(event);
    }
    ~RustObject() {
        auto r = ptr_qobject;
        ptr_qobject.invalidate();
        if (extra_destruct)
            extra_destruct(this);
        if (r.isValid())
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
