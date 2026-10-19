//! Native batch declarations are separated into mesh selections and instance behavior.
use super::super::Error;
use crate::model::profiles::{
    wmo::{DrawProperties, Lighting, Pass},
    Wmo,
};
use crate::model::{
    Bounds, Draw, DrawSubset, MaterialId, MeshId, ModelBuilder, PrimitiveRange, PrimitiveSelection,
    StripSelection, TangentBinding, Texture,
};
/// Complete world-surface draw controls independent of batch record revisions.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ImportedDrawControls {
    /// Purpose of this draw.
    pub pass: Pass,
    /// Illumination class for this draw.
    pub lighting: Lighting,
    /// Authored culling bounds when explicitly provided.
    pub bounds: Option<Bounds>,
    /// Optional baked-light texture resource.
    pub lightmap: Option<Texture>,
    /// Historical baked-light selector whose table interpretation remains unresolved.
    pub lightmap_selector: Option<u8>,
    /// Inclusive referenced vertex window, when explicitly authored.
    pub vertex_window: Option<[u32; 2]>,
    /// Optional strip connectivity selected from the owning geometry's strip stream.
    pub strip: Option<StripWindow>,
    /// Optional tangent vectors selected from the owning geometry's tangent pool.
    pub tangents: Option<TangentWindow>,
    /// Individual unresolved supplemental controls surrounding the draw selection.
    pub supplemental: [u16; 5],
    /// Historical alignment control whose meaning remains unresolved.
    pub alignment: u8,
}
/// An authored window into shared strip connectivity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct StripWindow {
    /// First strip index in the owning geometry's shared strip stream.
    pub first: u32,
    /// Number of strip index elements selected by this draw.
    pub count: u32,
    /// Unresolved supplemental strip control.
    pub control: u16,
}
/// An authored tangent start and the draw's referenced vertex window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TangentWindow {
    /// First vertex in the draw's authored vertex window.
    pub first_vertex: u32,
    /// Number of consecutive vertices in the draw's authored vertex window.
    pub vertex_count: u32,
    /// First tangent vector in the owning geometry's shared tangent pool.
    pub first_tangent: u32,
}

pub(crate) fn bind_draw(
    builder: &mut ModelBuilder<'_, Wmo>,
    mesh_id: MeshId,
    first: u32,
    count: u32,
    material: MaterialId,
    controls: Option<ImportedDrawControls>,
) -> Result<(Draw<Wmo>, MaterialId), Error> {
    let mut mesh = builder
        .model()
        .mesh(mesh_id)
        .expect("imported mesh")
        .clone();
    let mut subset = DrawSubset::new(PrimitiveSelection {
        connectivity: mesh.subsets.primary(),
        range: PrimitiveRange::Primitives { first, count },
    });
    let controls = controls
        .map(|value| {
            subset.vertex_window = value.vertex_window;
            subset.tangents = value.tangents.map(|value| TangentBinding {
                first_vertex: value.first_vertex,
                vertex_count: value.vertex_count,
                first_tangent: value.first_tangent,
            });
            if let Some(value) = value.strip {
                subset.strip = Some(StripSelection {
                    selection: PrimitiveSelection {
                        connectivity: mesh.connectivity_id(1)?,
                        range: PrimitiveRange::StripIndices {
                            first: value.first,
                            count: value.count,
                        },
                    },
                    control: value.control,
                });
            }
            Ok::<_, Error>(DrawProperties {
                pass: value.pass,
                lighting: value.lighting,
                bounds: value.bounds,
                lightmap: value.lightmap,
                lightmap_selector: value.lightmap_selector,
                supplemental: value.supplemental,
                alignment: value.alignment,
            })
        })
        .transpose()?;
    let subset = mesh.add_subset(subset);
    builder.set_mesh(mesh_id, mesh)?;
    Ok((Draw::new(subset, controls), material))
}
