//! Shared vertex storage with constrained profile owners and a profile-independent view.
use super::geometry_edit::remap_values;
use super::{Influence, Topology, Values};
use std::ops::{Deref, DerefMut};

/// Joint distributions preserve their authored slot structure and blending interpretation.
#[derive(Clone, Debug, Default)]
pub enum JointInfluences<'data> {
    /// No authored skinning distribution.
    #[default]
    None,
    /// Equal blending among the ordered joints of each matrix group.
    MatrixGroups(Values<'data, Vec<u32>>),
    /// Four ordered weighted joint slots, including zero-weight slots.
    Four(Values<'data, [Influence; 4]>),
}
/// Typed vertex streams shared by constrained geometry owners. Stream types
/// express fixed UV slots, independent vector tangents, multiple colors, or
/// unavailable streams. Additional vertex data belongs to the explicit extras type.
#[derive(Clone, Debug, Default)]
pub struct VertexData<
    'data,
    U = Vec<Values<'data, [f32; 2]>>,
    T = Values<'data, [f32; 4]>,
    C = Values<'data, [f32; 4]>,
    N = Values<'data, [f32; 3]>,
    E = (),
> {
    /// Model-space positions.
    pub positions: Values<'data, [f32; 3]>,
    /// Normal stream, or () for geometry without shading.
    pub normals: N,
    /// Tangent representation supported by this vocabulary.
    pub tangents: T,
    /// Primary or independently ordered color streams.
    pub colors: C,
    /// Ordered texture-coordinate streams or fixed slots.
    pub texture_coordinates: U,
    /// Additional independently authored vertex streams.
    pub extras: E,
}

/// Mechanical vertex correspondence for one typed stream or stream collection.
pub(crate) trait RemapStreams {
    fn remap_streams(&mut self, indices: &[u32]);
}
impl RemapStreams for () {
    fn remap_streams(&mut self, _: &[u32]) {}
}
impl<'a, T: Clone + Send + Sync + 'a> RemapStreams for Values<'a, T> {
    fn remap_streams(&mut self, indices: &[u32]) {
        remap_values(self, indices);
    }
}
impl<'a, T: Clone + Send + Sync + 'a> RemapStreams for Vec<Values<'a, T>> {
    fn remap_streams(&mut self, indices: &[u32]) {
        for stream in self {
            stream.remap_streams(indices);
        }
    }
}
impl<'a, T: Clone + Send + Sync + 'a, const N: usize> RemapStreams for [Values<'a, T>; N] {
    fn remap_streams(&mut self, indices: &[u32]) {
        for stream in self {
            stream.remap_streams(indices);
        }
    }
}
impl<U, T, C, N, E> VertexData<'_, U, T, C, N, E> {
    pub(crate) fn remap(&mut self, indices: &[u32])
    where
        U: RemapStreams,
        T: RemapStreams,
        C: RemapStreams,
        N: RemapStreams,
        E: RemapStreams,
    {
        self.positions.remap_streams(indices);
        self.normals.remap_streams(indices);
        self.tangents.remap_streams(indices);
        self.colors.remap_streams(indices);
        self.texture_coordinates.remap_streams(indices);
        self.extras.remap_streams(indices);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct GeometryStreams<'data, V, I> {
    pub vertices: V,
    pub topology: Topology<'data>,
    pub influences: I,
}
impl<V: Default, I: Default> Default for GeometryStreams<'_, V, I> {
    fn default() -> Self {
        Self {
            vertices: V::default(),
            topology: Topology::Triangles(Values::default()),
            influences: I::default(),
        }
    }
}
impl<V, I> Deref for GeometryStreams<'_, V, I> {
    type Target = V;
    fn deref(&self) -> &V {
        &self.vertices
    }
}
impl<V, I> DerefMut for GeometryStreams<'_, V, I> {
    fn deref_mut(&mut self) -> &mut V {
        &mut self.vertices
    }
}
pub(crate) type VertexStreams<
    'data,
    I,
    U = Vec<Values<'data, [f32; 2]>>,
    T = Values<'data, [f32; 4]>,
    C = Values<'data, [f32; 4]>,
    N = Values<'data, [f32; 3]>,
    E = (),
> = GeometryStreams<'data, VertexData<'data, U, T, C, N, E>, I>;
