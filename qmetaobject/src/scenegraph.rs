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
use super::*;
use cpp::cpp;
use std::hash::{Hash, Hasher};

/// A typed node in the scene graph
///
/// the SGNode owns a QSGNode* of a given type. The type information is given by T
/// which is a Tag type  (an empty enum)
#[repr(C)]
pub struct SGNode<T> {
    pub raw: *mut c_void,
    _phantom: std::marker::PhantomData<T>,
}
impl<T> SGNode<T> {
    pub unsafe fn from_raw(raw: *mut c_void) -> Self {
        /*let t = T::TYPE;
        debug_assert!(cpp!([raw as "QSGNode*", t as "QSGNode::NodeType"]
            -> bool as "bool" { return raw->type() == t; }));*/
        Self { raw, _phantom: Default::default() }
    }

    /// "leak" the QSGNode* pointer, so the caller must take ownership
    pub fn into_raw(self) -> *mut c_void {
        let cpp = self.raw;
        std::mem::forget(self);
        cpp
    }

    // Delete this node
    pub fn reset(&mut self) {
        *self = unsafe { Self::from_raw(std::ptr::null_mut()) };
    }
}
impl<T> Drop for SGNode<T> {
    /// Destroy the SGNode*
    fn drop(&mut self) {
        let raw = self.raw;
        cpp!(unsafe [raw as "QSGNode*"] { delete raw; });
    }
}

/// Tag to be used in SGNode. SGNode<ContainerNode> is a node that simply contains other node.
/// Either all the node have the same type, but the number of nodes is not known at compile time,
/// or the child node can have different type, but the amount of nodes is known at compile time
pub enum ContainerNode {}

cpp! {{
    struct ContainerNode : QSGNode {
        quint64 type_id = 0;
        std::size_t size = 0; // -1 for static
        quint64 mask = 0; // one bit for every child, if it is set, or not
        ContainerNode(quint64 type_id, std::size_t size) : type_id(type_id), size(size) {}
    };
}}

/// Represent a tuple of `Fn(`[`SGNode`]`<...>) -> SGNode<...>)`,
/// for [`SGNode<ContainerNode>::update_static`].
///
/// Do not reimplement
#[cfg_attr(feature = "cargo-clippy", allow(clippy::len_without_is_empty))]
pub trait UpdateNodeFnTuple<T> {
    fn len(&self) -> u64;
    unsafe fn update_fn(&self, i: u64, _: *mut c_void) -> *mut c_void;
}

// Implementation for tuple of different sizes
macro_rules! declare_UpdateNodeFnTuple {
    (@continue $T:ident : $A:ident : $N:tt $( $tail:tt )+) => { declare_UpdateNodeFnTuple![$( $tail )*]; };
    (@continue $T:ident : $A:ident : $N:tt) => {};
    ($( $T:ident : $A:ident : $N:tt )+) => {
        impl<$( $A, $T: Fn(SGNode<$A>) -> SGNode<$A> ),*> UpdateNodeFnTuple<($( $A, )*)> for ($( $T, )*)
        {
            fn len(&self) -> u64 { ($( $N, )* ).0 + 1 }
            unsafe fn update_fn(&self, i: u64, n: *mut c_void) -> *mut c_void {
                match i {
                    $(
                        $N => (self.$N)( SGNode::<_>::from_raw(n) ).into_raw(),
                    )*
                    _ => panic!("Out of range") }
            }
        }

        declare_UpdateNodeFnTuple![@continue $( $T : $A : $N )*];
    }
}
declare_UpdateNodeFnTuple![T9:A9:9 T8:A8:8 T7:A7:7 T6:A6:6 T5:A5:5 T4:A4:4 T3:A3:3 T2:A2:2 T1:A1:1 T0:A0:0];

// Implementation for single element
impl<A, T: Fn(SGNode<A>) -> SGNode<A>> UpdateNodeFnTuple<(A,)> for T {
    fn len(&self) -> u64 {
        1
    }
    unsafe fn update_fn(&self, _i: u64, n: *mut c_void) -> *mut c_void {
        self(SGNode::<A>::from_raw(n)).into_raw()
    }
}

impl SGNode<ContainerNode> {
    /// Update the child nodes from an iterator.
    ///
    /// When calling this function, all child nodes must be of the same type, but the amount
    /// of node must not be known at compile time.
    ///
    /// The array must have the same size every time the function is called on the node.
    /// (panic otherwise).
    /// call reset() if you want to change the size.
    ///
    /// ```
    /// # use qmetaobject::{QObject, qtdeclarative::QQuickItem};
    /// use qmetaobject::scenegraph::{SGNode, ContainerNode, RectangleNode};
    /// use qttypes::QRectF;
    ///
    /// # struct Dummy<T> { items: Vec<QRectF>, _phantom: T }
    /// # impl<T> QQuickItem for Dummy<T> where Dummy<T> : QObject  {
    /// // in the reimplementation of  QQuickItem::update_paint_node
    /// fn update_paint_node(&mut self, mut node: SGNode<ContainerNode>) -> SGNode<ContainerNode> {
    ///    let items: &Vec<QRectF> = &self.items;
    ///    node.update_dynamic(items.iter(),
    ///        |i, mut n| -> SGNode<RectangleNode> {
    ///            n.create(self);
    ///            n.set_rect(*i);
    ///            n
    ///        });
    ///    node
    ///  }
    /// # }
    /// ```
    pub fn update_dynamic<T: std::any::Any, Iter: ExactSizeIterator, F>(
        &mut self,
        iter: Iter,
        mut f: F,
    ) where
        F: FnMut(<Iter as Iterator>::Item, SGNode<T>) -> SGNode<T>,
    {
        let mut raw = self.raw;
        let type_id = get_type_hash::<T>();
        let len = iter.len();
        assert!(len <= 64, "There is a limit of 64 child nodes");
        let mut mask = 0u64;
        if raw.is_null() {
            raw = cpp!(unsafe [type_id as "quint64", len as "std::size_t"] -> *mut c_void as "QSGNode*" {
                return new ContainerNode(type_id, len);
            });
            self.raw = raw;
        } else {
            mask = cpp!(unsafe [raw as "ContainerNode*", type_id as "quint64", len as "std::size_t"] -> u64 as "quint64" {
                if (raw->size != len || raw->type_id != type_id) {
                    rust!(sgnode_0 []{ panic!("update_dynamic must always be called with the same type and the same number of elements") });
                }
                return raw->mask;
            });
        }
        let mut bit = 1u64;
        let mut before_iter: *mut c_void = std::ptr::null_mut();
        for i in iter {
            before_iter = Self::iteration(raw, before_iter, &mut bit, &mut mask, |n| {
                f(i, unsafe { SGNode::<T>::from_raw(n) }).into_raw()
            });
        }
        cpp!(unsafe [raw as "ContainerNode*", mask as "quint64"] {
            raw->mask = mask;
        });
    }

    /// Update the child node: given a tuple of update function, runs it for every node
    ///
    /// The argument is a tuple of update functions. The same node types must be used every time
    /// this function is called. (If reset() was not called in between)
    /// (Panic otherwise).
    /// Each node type can be different.
    ///
    /// In this example, the node has two children node
    ///
    /// ```
    /// # use qmetaobject::{QObject, qtdeclarative::QQuickItem};
    /// use qmetaobject::scenegraph::{SGNode, ContainerNode, RectangleNode};
    /// use qttypes::QRectF;
    ///
    /// # struct Dummy<T> { items: Vec<QRectF>, _phantom: T }
    /// # impl<T> QQuickItem for Dummy<T> where Dummy<T> : QObject  {
    /// // in the reimplementation of  QQuickItem::update_paint_node
    /// fn update_paint_node(&mut self, mut node : SGNode<ContainerNode> ) -> SGNode<ContainerNode> {
    ///      node.update_static((
    ///          |mut n : SGNode<RectangleNode>| -> SGNode<RectangleNode> {
    ///              n.create(self);
    ///              n.set_rect(QRectF { x: 0., y: 0., width: 42., height: 42. });
    ///              n
    ///          },
    ///          |mut n : SGNode<RectangleNode>| -> SGNode<RectangleNode> {
    ///              n.create(self);
    ///              n.set_rect(QRectF { x: 0., y: 0., width: 42., height: 42. });
    ///              n
    ///          }));
    ///      node
    ///  }
    /// # }
    ///  ```
    pub fn update_static<A: 'static, T: UpdateNodeFnTuple<A>>(&mut self, info: T) {
        let type_id = get_type_hash::<A>();
        let mut mask = 0u64;
        if self.raw.is_null() {
            self.raw = cpp!(unsafe [type_id as "quint64"] -> *mut c_void as "QSGNode*" {
                return new ContainerNode(type_id, -1);
            });
        } else {
            let raw = self.raw;
            mask = cpp!(unsafe [raw as "ContainerNode*", type_id as "quint64"] -> u64 as "quint64" {
                if (raw->size != std::size_t(-1) || raw->type_id != type_id) {
                    rust!(sgnode_3 []{ panic!("update_static must always be called with the same type of functions") });
                }
                return raw->mask;
            });
        }
        let mut bit = 1u64;
        let mut before_iter: *mut c_void = std::ptr::null_mut();
        for i in 0..info.len() {
            before_iter = Self::iteration(self.raw, before_iter, &mut bit, &mut mask, |n| unsafe {
                info.update_fn(i, n)
            });
        }
        let raw_ = self.raw;
        cpp!(unsafe [raw_ as "ContainerNode*", mask as "quint64"] {
            raw_->mask = mask;
        });
    }

    // returns the new before_iter
    fn iteration<F: FnOnce(*mut c_void) -> *mut c_void>(
        raw: *mut c_void,
        before_iter: *mut c_void,
        bit: &mut u64,
        mask: &mut u64,
        update_fn: F,
    ) -> *mut c_void {
        let node = if (*mask & *bit) == 0 {
            std::ptr::null_mut()
        } else {
            cpp!(unsafe [raw as "QSGNode*", before_iter as "QSGNode*"] -> *mut c_void as "QSGNode*" {
                auto node = before_iter ? before_iter->nextSibling() : raw->firstChild();
                if (!node) rust!(sgnode_2 []{ panic!("There must be a node as the mask says so") });
                node->setFlag(QSGNode::OwnedByParent, false); // now we own it;
                return node;
            })
        };
        let node = update_fn(node);
        *mask = if node.is_null() { *mask & !*bit } else { *mask | *bit };
        if !node.is_null() {
            cpp!(unsafe [raw as "QSGNode*", node as "QSGNode*", before_iter as "QSGNode*"] {
                if (!node->parent()) {
                    if (before_iter)
                        raw->insertChildNodeAfter(node, before_iter);
                    else
                        raw->prependChildNode(node);
                } else if (node->parent() != raw) {
                    rust!(sgnode_4 []{ panic!("Returned node from another parent") });
                }
                node->setFlag(QSGNode::OwnedByParent);
            });
        }
        (*bit) <<= 1;
        if node.is_null() {
            before_iter
        } else {
            node
        }
    }
}

fn get_type_hash<T: std::any::Any>() -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::any::TypeId::of::<T>().hash(&mut hasher);
    hasher.finish()
}

/*

pub trait UpdateNodeInfo {
    fn create(&self) -> SGNode;
    fn update(&self, &mut SGNode);
}

pub trait NodeType {
    fn from_node(&mut SGNode) -> &mut Self;
    fn into_node(self) -> SGNode;
}

pub struct UpdateNode<T: NodeType, Fn1: Fn() -> T, Fn2: Fn(&mut T)> {
    pub create: Fn1,
    pub update: Fn2,
}

impl<T: NodeType, Fn1: Fn() -> T, Fn2: Fn(&mut T)> UpdateNodeInfo for UpdateNode<T, Fn1, Fn2> {
    fn create(&self) -> SGNode {
        (self.create)().into_node()
    }
    fn update(&self, n: &mut SGNode) {
        (self.update)(T::from_node(n));
    }
}

impl SGNode {
    pub fn update_nodes<'a>(&mut self, info: &[&'a UpdateNodeInfo])
    {
        if self.raw.is_null() {
            self.raw = cpp!(unsafe [] -> *mut c_void as "QSGNode*" {
                return new QSGNode;
            });
        }
        let raw = self.raw;
        let mut iter = cpp!(unsafe [raw as "QSGNode*"] -> *mut c_void as "QSGNode*" {
            return raw->firstChild();
        });
        for update_info in info {
            if iter.is_null() {
                let mut new_node = update_info.create();
                update_info.update(&mut new_node);
                iter = new_node.raw;
                cpp!(unsafe [raw as "QSGNode*", iter as "QSGNode*" ] {
                    raw->appendChildNode(iter);
                });
                std::mem::forget(new_node);
            } else {
                update_info.update(unsafe {
                    // a reference to a void* is the same as a reference to a SGNode
                    std::mem::transmute::<&mut (*mut c_void), &mut SGNode>(&mut iter)
                });
            }

            iter = cpp!(unsafe [iter as "QSGNode*"] -> *mut c_void as "QSGNode*" {
                return iter->nextSibling();
            });
        }
    }
}
*/

#[cfg(qt_5_8)]
/// Wrapper around QSGRectangleNode
pub enum RectangleNode {}

cpp! {{
    // Just a stub for compatibility
    #if QT_VERSION < QT_VERSION_CHECK(5, 8, 0)
    struct QSGRectangleNode{};
    #endif
}}

#[cfg(qt_5_8)]
impl SGNode<RectangleNode> {
    pub fn set_color(&mut self, color: QColor) {
        let raw = self.raw;
        cpp!(unsafe [raw as "QSGRectangleNode*", color as "QColor"] {
            #if QT_VERSION >= QT_VERSION_CHECK(5, 8, 0)
            if(raw) raw->setColor(color);
            #endif
        });
    }
    pub fn set_rect(&mut self, rect: QRectF) {
        let raw = self.raw;
        cpp!(unsafe [raw as "QSGRectangleNode*", rect as "QRectF"] {
            #if QT_VERSION >= QT_VERSION_CHECK(5, 8, 0)
            if (raw) raw->setRect(rect);
            #endif
        });
    }

    pub fn create(&mut self, item: &dyn QQuickItem) {
        if !self.raw.is_null() {
            return;
        }
        let item = item.get_cpp_object();
        self.raw = cpp!(unsafe [item as "QQuickItem*"] -> *mut c_void as "void*" {
            #if QT_VERSION >= QT_VERSION_CHECK(5, 8, 0)
            if (!item) return nullptr;
            if (auto window = item->window())
                return window->createRectangleNode();
            #endif
            return nullptr;
        });
    }
}

/// Wrapper around QSGTransformNode
pub enum TransformNode {}

impl SGNode<TransformNode> {
    pub fn set_translation(&mut self, x: f64, y: f64) {
        if self.raw.is_null() {
            self.create();
        }
        let raw = self.raw;
        cpp!(unsafe [raw as "QSGTransformNode*", x as "double", y as "double"] {
            QMatrix4x4 m;
            m.translate(x, y);
            if (raw) raw->setMatrix(m);
        });
    }

    pub fn create(&mut self) {
        if !self.raw.is_null() {
            return;
        }
        self.raw = cpp!(unsafe [] -> *mut c_void as "void*" { return new QSGTransformNode; });
    }

    pub fn update_sub_node<F: FnMut(SGNode<ContainerNode>) -> SGNode<ContainerNode>>(
        &mut self,
        mut f: F,
    ) {
        if self.raw.is_null() {
            self.create();
        }
        let raw = self.raw;
        let sub = unsafe {
            SGNode::<ContainerNode>::from_raw(
                cpp!([raw as "QSGNode*"] -> *mut c_void as "QSGNode*" {
                    auto n = raw->firstChild();
                    if (n)
                        n->setFlag(QSGNode::OwnedByParent, false); // now we own it;
                    return n;
                }),
            )
        };
        let sub = f(sub);
        let node = sub.into_raw();
        cpp!(unsafe [node as "QSGNode*", raw as "QSGNode*"] {
            if (!node->parent()) {
                raw->prependChildNode(node);
            } else if (node->parent() != raw) {
                rust!(sgnode_5 []{ panic!("Returned node from another parent") });
            }
            node->setFlag(QSGNode::OwnedByParent);
        });
    }
}

/*
#[repr(C)]
struct SGGeometryNode {
    node : SGNode,
}

cpp! {{
struct RustGeometryNode : QSGGeometryNode {
    QSGGeometry geo;
    RustGeometryNode(const QSGGeometry::AttributeSet &attribs, int vertexCount)
        : geo(attribs, vertexCount) {
        setGeometry(&geo);
    }
}
}}

impl SGGeometryNode<> {
    fn create() -> Self {
        Self { node : SGNode { raw : cpp!(unsafe [] -> c_void as "QSGNode*" {
            return new RustGeometryNode()
        })}}
    }
}*/

/*



pub trait SGNodeType {
    const TYPE : u32;
}
enum BasicNodeType {}
impl SGNodeType for BasicNodeType { const TYPE : u32 = 0; }
enum GeometryNodeType {}
impl SGNodeType for GeometryNodeType { const TYPE : u32 = 1; }

pub struct SGNodeBox {
    cpp: *mut c_void,
}
impl SGNodeBox {
    pub unsafe fn from_raw(raw : *mut c_void) -> Self {
        SGNodeBox { cpp: raw }
    }
    pub fn into_raw(self) -> *mut c_void {
        let cpp = self.cpp;
        std::mem::forget(self);
        cpp
    }

    pub fn as_ref<'a, T: SGNodeType>(&'a mut self)-> SGNodeRef<'a, T> {
        unsafe { SGNodeRef::from_raw(self.cpp) }
    }
}
impl Drop for SGNodeBox {
    fn drop(&mut self) {
        let cpp = self.cpp;
        cpp!(unsafe [cpp as "QSGNode*"] { delete cpp; });
    }
}

pub struct SGNodeRef<'a, T : SGNodeType + 'a> {
    cpp: *mut c_void,
    _phantom: std::marker::PhantomData<&'a T>,
}
impl<'a, T : SGNodeType> SGNodeRef<'a, T> {
    pub unsafe fn from_raw(raw : *mut c_void) -> Self {
        let t = T::TYPE;
        debug_assert!(cpp!([raw as "QSGNode*", t as "QSGNode::NodeType"]
            -> bool as "bool" { return raw->type() == t; }));
        Self { cpp: raw, _phantom: Default::default() }
    }
    pub fn raw(&self) -> *mut c_void {
        self.cpp
    }

    pub fn append_child_node(&mut self, node : SGNodeBox) {
        let cpp = self.cpp;
        let n = node.into_raw();
        cpp!(unsafe [cpp as "QSGNode*", n as "QSGNode*"] { cpp->appendChildNode(n); });
    }

    pub fn child_at_index<T2>(&'a self, i : u32) -> SGNodeRef<'a, T> {
        let cpp = self.cpp;
        unsafe { SGNodeRef::from_raw(cpp!([cpp as "QSGNode*", i as "int"]
            -> *mut c_void as "QSGNode*" { return cpp->childAtIndex(i); })) }
    }

    pub fn mark_dirty(&mut self, bits : DirtyState ) {
        let cpp = self.cpp;
        cpp!(unsafe [cpp as "QSGNode*", bits as "QSGNode::DirtyState"]
            { return cpp->markDirty(bits); });
    }
}


bitflags! {
    /// Maps to Qt's QSGNode::DirtyState
    pub struct DirtyState : u32 {
        const SUBTREE_BLOCKED         = 0x0080;
        const MATRIX                 = 0x0100;
        const NODE_ADDED              = 0x0400;
        const NODE_REMOVED            = 0x0800;
        const GEOMETRY               = 0x1000;
        const MATERIAL               = 0x2000;
        const OPACITY                = 0x4000;
        const FORCE_UPDATE            = 0x8000;
/*
        const USE_PREPROCESS          = 0x0002 /*UsePreprocess*/;
        const PROPAGATION_MASK        = Self::MATRIX.bits
                                      | Self::NODE_ADDED.bits
                                      | Self::OPACITY.bits
                                      | Self::FORCE_UPDATE.bits;*/
    }
}




bitflags! {
    /// Maps to Qt's QQuickWindow::CreateTextureOption
    pub struct CreateTextureOption : u32 {
        const TEXTURE_HAS_ALPHA_CHANNEL = 0x0001;
        const TEXTURE_HAS_MIPMAPS       = 0x0002;
        const TEXTURE_OWNS_GLTEXTURE    = 0x0004;
        const TEXTURE_CAN_USE_ATLAS     = 0x0008;
        const TEXTURE_IS_OPAQUE         = 0x0010;
    }
}

pub fn create_texture_from_image(item : &QQuickItem, image : &QImage,
                                 options : CreateTextureOption)
                                 -> SGTextureBox {
    let item = item.get_cpp_object();
    cpp!(unsafe [item as "QQuickItem*", image as "QImage*", options as "QQuickWindow::CreateTextureOption"]
            -> SGTextureBox as "QSGTexture*" {
        if (!item) return nullptr;
        if (auto window = item->window())
            return window->createTextureFromImage(*image, options);
        return nullptr;
    })
}


*/
