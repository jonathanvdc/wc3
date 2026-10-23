//! Bevy-owned data slabs; the public material remains an ordinary animated asset.
use super::{CombinerUniform, HdUniform, ModelMaterialExtension, WorldUniform};
use bevy::prelude::{Handle, Image};
use bevy::render::render_resource::{AsBindGroup, ShaderType};

/// A single storage-array element, not an array of individual buffer bindings.
#[derive(Clone, Copy, ShaderType)]
pub(super) struct MaterialData {
    pub(super) combiner: CombinerUniform,
    pub(super) hd: HdUniform,
    pub(super) world: WorldUniform,
}
#[derive(AsBindGroup)]
#[data(50, MaterialData, binding_array(116))]
#[bindless(index_table(range(50..63), binding(100)))]
pub(super) struct BindlessSnapshot {
    data: MaterialData,
    #[texture(51)]
    #[sampler(52)]
    secondary: Option<Handle<Image>>,
    #[texture(53)]
    #[sampler(54)]
    orm: Option<Handle<Image>>,
    #[texture(55)]
    #[sampler(56)]
    team_color: Option<Handle<Image>>,
    #[texture(57)]
    #[sampler(58)]
    environment: Option<Handle<Image>>,
    #[texture(59)]
    #[sampler(60)]
    tertiary: Option<Handle<Image>>,
    #[texture(61)]
    #[sampler(62)]
    fourth: Option<Handle<Image>>,
}
impl From<&BindlessSnapshot> for MaterialData {
    fn from(snapshot: &BindlessSnapshot) -> Self {
        snapshot.data
    }
}
impl From<&ModelMaterialExtension> for BindlessSnapshot {
    fn from(material: &ModelMaterialExtension) -> Self {
        Self {
            data: MaterialData {
                combiner: material.combiner,
                hd: material.hd,
                world: material.world,
            },
            secondary: material.secondary.clone(),
            orm: material.orm.clone(),
            team_color: material.team_color.clone(),
            environment: material.environment.clone(),
            tertiary: material.tertiary.clone(),
            fourth: material.fourth.clone(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::super::encode_uniform;
    use super::*;
    use bevy::pbr::{ExtendedMaterial, StandardMaterial};
    use bevy::render::render_resource::BindlessResourceType;
    #[test]
    fn descriptor_uses_one_data_buffer_and_disjoint_extension_indices() {
        let d = BindlessSnapshot::bindless_descriptor().unwrap();
        assert_eq!(d.resources.len(), 63);
        assert!(d.resources[..50]
            .iter()
            .all(|v| *v == BindlessResourceType::None));
        assert_eq!(d.resources[50], BindlessResourceType::DataBuffer);
        assert!(!d.resources.contains(&BindlessResourceType::Buffer));
        assert_eq!(d.buffers.len(), 1);
        assert_eq!(d.buffers[0].binding_number.0, 116);
        assert_eq!(d.buffers[0].bindless_index.0, 50);
        assert_eq!(
            d.buffers[0].size.unwrap() as u64,
            MaterialData::min_size().get()
        );
        assert_eq!(d.index_tables.len(), 1);
        assert_eq!(d.index_tables[0].indices.start.0, 50);
        assert_eq!(d.index_tables[0].indices.end.0, 63);
        assert_eq!(d.index_tables[0].binding_number.0, 100);
        let merged =
            <ExtendedMaterial<StandardMaterial, ModelMaterialExtension>>::bindless_descriptor()
                .unwrap();
        assert_eq!(merged.index_tables.len(), 2);
    }
    #[test]
    fn snapshot_keeps_public_material_values_and_image_handles() {
        let mut m = ModelMaterialExtension::default();
        m.combiner.weights.y = 0.375;
        m.hd.fresnel.x = 2.;
        m.world.ambient.z = 0.125;
        let snapshot = BindlessSnapshot::from(&m);
        let data = MaterialData::from(&snapshot);
        assert_eq!(data.combiner.weights.y, 0.375);
        assert_eq!(data.hd.fresnel.x, 2.);
        assert_eq!(data.world.ambient.z, 0.125);
        assert_eq!(snapshot.secondary, m.secondary);
        let data_bytes = encode_uniform(&data);
        let mut concatenated = encode_uniform(&m.combiner);
        concatenated.extend(encode_uniform(&m.hd));
        concatenated.extend(encode_uniform(&m.world));
        assert_eq!(
            data_bytes, concatenated,
            "combined storage row retains nested shader field layout"
        );
    }
}
