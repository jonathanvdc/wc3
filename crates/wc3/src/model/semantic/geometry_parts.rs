//! Adapter extraction moves canonical attributes into constrained stream owners.
use super::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivitySets, Error,
    GeometryData, GeometryInfluences, GeometryKind, GeometryPools, GeometrySchema,
    SurfaceTriangleControls, Topology, ZeroVec,
};

#[derive(Clone, Debug)]
pub(crate) struct GeometryParts<'data> {
    pub kind: GeometryKind,
    pub strip_set: bool,
    pub positions: ZeroVec<'data, [f32; 3]>,
    pub normals: ZeroVec<'data, [f32; 3]>,
    pub tangents: ZeroVec<'data, [f32; 4]>,
    pub colors: ZeroVec<'data, [f32; 4]>,
    pub texture_coordinates: Vec<ZeroVec<'data, [f32; 2]>>,
    pub topology: Topology<'data>,
    pub influences: GeometryInfluences<'data>,
    pub attributes: Vec<Attribute<'data>>,
    pub surface_triangles: ZeroVec<'data, SurfaceTriangleControls>,
    pub pools: GeometryPools<'data>,
}
impl Default for GeometryParts<'_> {
    fn default() -> Self {
        Self {
            kind: GeometryKind::Display,
            strip_set: false,
            positions: ZeroVec::default(),
            normals: ZeroVec::default(),
            tangents: ZeroVec::default(),
            colors: ZeroVec::default(),
            texture_coordinates: Vec::new(),
            topology: Topology::Triangles(ZeroVec::default()),
            influences: GeometryInfluences::None,
            attributes: Vec::new(),
            surface_triangles: ZeroVec::default(),
            pools: GeometryPools::default(),
        }
    }
}
impl<'data> GeometryParts<'data> {
    pub(crate) fn into_data(self) -> Result<GeometryData<'data>, Error> {
        let value = self;
        let mut result = GeometryData {
            kind: value.kind,
            positions: value.positions,
            connectivity: ConnectivitySets::new(
                value.topology,
                if value.strip_set || !value.pools.triangle_strip_indices.is_empty() {
                    vec![Topology::TriangleStrip(value.pools.triangle_strip_indices)]
                } else {
                    Vec::new()
                },
            ),
            influences: value.influences,
            ..GeometryData::default()
        };
        result.set_normals(value.normals);
        result.set_tangents(value.tangents);
        result.set_colors(value.colors);
        result.set_texture_coordinates(value.texture_coordinates);
        for attribute in value.attributes {
            result.attributes.insert(attribute)?;
        }
        if !value.surface_triangles.is_empty() {
            result.attributes.insert(Attribute {
                semantic: AttributeSemantic::SurfaceControls,
                domain: AttributeDomain::Face,
                values: AttributeValues::SurfaceControls(value.surface_triangles),
            })?;
        }
        if !value.pools.tangent_vectors.is_empty() {
            result.attributes.insert(Attribute {
                semantic: AttributeSemantic::Tangent,
                domain: AttributeDomain::Pool,
                values: AttributeValues::Vector4(value.pools.tangent_vectors),
            })?;
        }
        Ok(result)
    }
}
impl<'data> GeometryData<'data> {
    pub(crate) fn into_parts(mut self) -> GeometryParts<'data> {
        let normals = self.normals().cloned().unwrap_or_default();
        let tangents = self.tangents().cloned().unwrap_or_default();
        let colors = self.colors().cloned().unwrap_or_default();
        let texture_coordinates = self.texture_coordinates().to_vec();
        let surface_triangles = self.surface_triangles().cloned().unwrap_or_default();
        let tangent_vectors = self.tangent_pool().cloned().unwrap_or_default();
        let strip_indices = self.strip_indices().cloned().unwrap_or_default();
        self.attributes.retain(|attribute| {
            if attribute.semantic == AttributeSemantic::Color(0)
                && attribute.domain == AttributeDomain::Vertex
                && attribute.values.is_empty()
            {
                return true;
            }
            !matches!(
                (&attribute.semantic, attribute.domain),
                (
                    AttributeSemantic::Normal
                        | AttributeSemantic::Tangent
                        | AttributeSemantic::Color(0)
                        | AttributeSemantic::TexCoord(_),
                    AttributeDomain::Vertex
                ) | (AttributeSemantic::SurfaceControls, AttributeDomain::Face)
                    | (AttributeSemantic::Tangent, AttributeDomain::Pool)
            )
        });
        GeometryParts {
            strip_set: !self.connectivity.additional().is_empty(),
            kind: self.kind,
            positions: self.positions,
            normals,
            tangents,
            colors,
            texture_coordinates,
            topology: self.connectivity.primary,
            influences: self.influences,
            attributes: self.attributes.into_iter().collect(),
            surface_triangles,
            pools: GeometryPools {
                triangle_strip_indices: strip_indices,
                tangent_vectors,
            },
        }
    }
}
