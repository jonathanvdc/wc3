//! Shared owned runtime definitions and projections within those definitions.
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

trait Definition<T>: Send + Sync {
    fn get(&self) -> &T;
}
struct Projection<P, T> {
    parent: Shared<P>,
    select: fn(&P) -> &T,
}
impl<P: Send + Sync + 'static, T: Send + Sync + 'static> Definition<T> for Projection<P, T> {
    fn get(&self) -> &T {
        (self.select)(&self.parent)
    }
}
enum Storage<T> {
    Value(Arc<T>),
    Projection(Arc<dyn Definition<T>>),
}
/// A runtime value that can share an immutable source definition or a projected field.
/// Editing a shared definition makes an independent copy of that definition.
pub struct Shared<T> {
    storage: Storage<T>,
}
impl<T: Send + Sync + 'static> Shared<T> {
    #[cfg(test)]
    pub(crate) fn strong_count(&self) -> usize {
        match &self.storage {
            Storage::Value(value) => Arc::strong_count(value),
            Storage::Projection(value) => Arc::strong_count(value),
        }
    }
    /// Owns a runtime definition.
    pub fn new(value: T) -> Self {
        Self {
            storage: Storage::Value(Arc::new(value)),
        }
    }
    /// Shares a field of this definition without copying it.
    pub fn map<U: Send + Sync + 'static>(&self, select: fn(&T) -> &U) -> Shared<U> {
        Shared {
            storage: Storage::Projection(Arc::new(Projection {
                parent: self.clone(),
                select,
            })),
        }
    }
    pub(crate) fn select<U: Send + Sync + 'static>(
        &self,
        select: impl Fn(&T) -> &U + Send + Sync + 'static,
    ) -> Shared<U> {
        struct Selected<P, T, F> {
            parent: Shared<P>,
            select: F,
            marker: PhantomData<T>,
        }
        impl<P: Send + Sync + 'static, T: Send + Sync + 'static, F: Fn(&P) -> &T + Send + Sync>
            Definition<T> for Selected<P, T, F>
        {
            fn get(&self) -> &T {
                (self.select)(&self.parent)
            }
        }
        Shared {
            storage: Storage::Projection(Arc::new(Selected {
                parent: self.clone(),
                select,
                marker: Default::default(),
            })),
        }
    }
}
impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            storage: match &self.storage {
                Storage::Value(v) => Storage::Value(v.clone()),
                Storage::Projection(v) => Storage::Projection(v.clone()),
            },
        }
    }
}
impl<T> Deref for Shared<T> {
    type Target = T;
    fn deref(&self) -> &T {
        match &self.storage {
            Storage::Value(v) => v,
            Storage::Projection(v) => v.get(),
        }
    }
}
impl<T: Clone> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        if matches!(self.storage, Storage::Projection(_)) {
            self.storage = Storage::Value(Arc::new((**self).clone()))
        }
        match &mut self.storage {
            Storage::Value(v) => Arc::make_mut(v),
            _ => unreachable!(),
        }
    }
}
impl<T: Send + Sync + 'static> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
impl<T: Debug> Debug for Shared<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.deref().fmt(f)
    }
}

use wc3::model::mdlx::animation::Track;
use wc3::model::mdlx::scene::{Node, NodeFlagInterpretation, NodeFlags};
/// One node's metadata and shared animation definitions.
#[derive(Clone)]
pub(crate) struct RuntimeNode {
    pub(crate) object_id: u32,
    pub(crate) parent_id: u32,
    pub(crate) flags: NodeFlags,
    pub(crate) translation: Shared<Option<Track<'static, [f32; 3]>>>,
    pub(crate) rotation: Shared<Option<Track<'static, [f32; 4]>>>,
    pub(crate) scaling: Shared<Option<Track<'static, [f32; 3]>>>,
}
impl RuntimeNode {
    pub(super) fn new<F: NodeFlagInterpretation + Send + Sync + 'static>(
        node: Shared<Node<'static, F>>,
    ) -> Self {
        Self {
            object_id: node.object_id,
            parent_id: node.parent_id,
            flags: NodeFlags::from_bits(node.flags.bits()),
            translation: node.map(|node| &node.translation),
            rotation: node.map(|node| &node.rotation),
            scaling: node.map(|node| &node.scaling),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::mdlx::animation::ValueKeyframe;

    #[test]
    fn cloned_runtime_nodes_share_prepared_key_buffers() {
        let mut node = Node::new("root", 0).unwrap();
        node.translation = Some(
            Track::linear(
                [
                    ValueKeyframe {
                        frame: 0,
                        value: [1.0; 3],
                    },
                    ValueKeyframe {
                        frame: 100,
                        value: [2.0; 3],
                    },
                ],
                None,
            )
            .unwrap(),
        );
        let source = Shared::new(node);
        let first = vec![RuntimeNode::new(source.clone())];
        let second = first.clone();
        let pointer = first[0]
            .translation
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()
            .as_bytes()
            .as_ptr();
        drop(source);
        for nodes in [&first, &second] {
            let keys = nodes[0]
                .translation
                .as_ref()
                .unwrap()
                .linear_keys()
                .unwrap();
            assert_eq!(keys.as_bytes().as_ptr(), pointer);
        }
        let clone = first[0].translation.clone();
        assert_eq!(
            clone
                .as_ref()
                .unwrap()
                .linear_keys()
                .unwrap()
                .as_bytes()
                .as_ptr(),
            pointer
        );
    }

    #[test]
    fn editing_a_projection_copies_only_that_definition() {
        #[derive(Clone)]
        struct Definition {
            vertices: Vec<u32>,
            normals: Vec<u32>,
        }
        let source = Shared::new(Definition {
            vertices: vec![1, 2, 3],
            normals: vec![4, 5, 6],
        });
        let mut vertices = source.map(|source| &source.vertices);
        let normals = source.map(|source| &source.normals);
        assert_eq!(vertices.as_ptr(), source.vertices.as_ptr());
        vertices.push(7);
        assert_eq!(&*vertices, &[1, 2, 3, 7]);
        assert_eq!(source.vertices, [1, 2, 3]);
        assert_eq!(normals.as_ptr(), source.normals.as_ptr());
    }
}
