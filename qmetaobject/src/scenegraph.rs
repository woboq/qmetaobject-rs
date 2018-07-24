use super::*;
use std::os::raw::c_void;

#[repr(C)]
pub struct SGNode {
    pub // remove
    raw: *mut c_void,
}

pub trait UpdateNodeInfo {
    fn create(&self) -> SGNode;
    fn update(&self, &mut SGNode);
}

pub trait NodeType {
    fn from_node(&mut SGNode) -> &mut Self;
    fn into_node(self) -> SGNode;
}

pub struct UpdateNode<T : NodeType, Fn1 : Fn()->T, Fn2: Fn(&mut T)> {
    pub create : Fn1,
    pub update : Fn2,
}

impl<T : NodeType, Fn1 : Fn()->T, Fn2: Fn(&mut T)> UpdateNodeInfo for UpdateNode<T, Fn1, Fn2> {
    fn create(&self) -> SGNode {
        (self.create)().into_node()
    }
    fn update(&self, n : &mut SGNode) {
        (self.update)(T::from_node(n));
    }
}

impl SGNode {
    pub fn update_nodes<'a>(&mut self, info : &[&'a UpdateNodeInfo])
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
impl Drop for SGNode {
    fn drop(&mut self) {
        let raw = self.raw;
        cpp!(unsafe [raw as "QSGNode*"] { delete raw; });
    }
}
/*
#[repr(C)]
struct SGGeometryNode {
    node : SGNode,
}

cpp!{{
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


/*
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum {}
*/

#[repr(C)]
pub struct SGTextureBox {
    cpp : *mut c_void,
}
impl Drop for SGTextureBox {
    fn drop(&mut self) {
        let cpp = self.cpp;
        cpp!(unsafe [cpp as "QSGTexture*"] { delete cpp; });
    }
}
/*impl<'a> SGTextureRef<'a> {
    //pub fn anisotropyLevel() ->

}*/


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

/*
pub trait QSGNode {
    fn is_sub_tree_blocked(&self) -> bool { false }
    fn preprocess(&mut self) { }
}*/
/*
struct SGNodeHolder<T : QSGNode> {
    cpp: *mut std::os::raw::c_void,
    node: RefCell<T>,
}

struct BaseNode {
}
*/


cpp!{{
struct RustSGNode : QSGNode {
    TraitObject QSGNode_trait;
    ~RustSGNode() {
        /*TraitObject toDelete = QSGNode_trait;
        QSGNode_trait = {};
        if (toDelete) {
            rust!(RustSGNode_delete[mut QSGNode_trait : *mut QSGNode as "TraitObject"] {
                let b = unsafe { Box::from_raw(QSGNode_trait) };
            })
        }*/
    }
    /*bool isSubtreeBlocked() const override {
        return rust!(QSGNode_isSubtreeBlocked [QSGNode_trait : &QSGNode as "TraitObject"] -> bool as "bool" {
            return QSGNode_trait.is_sub_tree_blocked();
        });
    }
    void preprocess() override {
        return rust!(QSGNode_preprocess [QSGNode_trait : &mut QSGNode as "TraitObject"] {
            QSGNode_trait.preprocess()
        });
    }*/
};
}}

/*
cpp_trait!{ RustSGNode : "QSGNode",
    pub trait QSGNode {
        fn is_sub_tree_blocked(&self) -> bool for "bool isSubtreeBlocked() const" {
            return rust!(QSGNode_isSubtreeBlocked [QSGNode_trait : &QSGNode as "TraitObject"] -> bool {
                QSGNode_trait.is_sub_tree_blocked();
            );
        }
    }
}
*/


*/
