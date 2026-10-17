//! GPU binding layout independent of semantic image/sampler ownership.
use super::m3::layers::M3Uniform;
use super::{
    CombinerUniform, HdUniform, ModelMaterialExtension, ModelMaterialKey, ParticleUniform,
    WorldUniform,
};
use bevy::ecs::system::{lifetimeless::SRes, SystemParamItem};
use bevy::prelude::{Handle, Image};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupError, BindGroupLayout, BindGroupLayoutEntry, BindingType,
    OwnedBindingResource, SamplerBindingType, ShaderStages, UnpreparedBindGroup,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::GpuShaderBuffer;
use bevy::render::texture::{FallbackImage, GpuImage};

const SAMPLER_BINDINGS: [u32; 6] = [102, 105, 107, 109, 112, 115];

#[derive(AsBindGroup)]
struct BindingSnapshot {
    #[uniform(100)]
    combiner: CombinerUniform,
    #[texture(101)]
    #[sampler(102)]
    secondary: Option<Handle<Image>>,
    #[uniform(103)]
    hd: HdUniform,
    #[texture(104)]
    #[sampler(105)]
    orm: Option<Handle<Image>>,
    #[texture(106)]
    #[sampler(107)]
    team_color: Option<Handle<Image>>,
    #[texture(108)]
    #[sampler(109)]
    environment: Option<Handle<Image>>,
    #[texture(111)]
    #[sampler(112)]
    tertiary: Option<Handle<Image>>,
    #[texture(114)]
    #[sampler(115)]
    fourth: Option<Handle<Image>>,
    #[uniform(110)]
    world: WorldUniform,
    #[uniform(113)]
    particle: ParticleUniform,
    #[uniform(116)]
    m3: M3Uniform,
    #[texture(117)]
    normal_secondary: Option<Handle<Image>>,
    #[texture(119)]
    normal_secondary_mask: Option<Handle<Image>>,
    #[texture(121)]
    light_map: Option<Handle<Image>>,
    #[texture(123)]
    ambient_occlusion: Option<Handle<Image>>,
}
impl From<&ModelMaterialExtension> for BindingSnapshot {
    fn from(material: &ModelMaterialExtension) -> Self {
        Self {
            combiner: material.combiner,
            secondary: material.secondary.clone(),
            hd: material.hd,
            orm: material.orm.clone(),
            team_color: material.team_color.clone(),
            environment: material.environment.clone(),
            tertiary: material.tertiary.clone(),
            fourth: material.fourth.clone(),
            world: material.world,
            particle: material.particle,
            m3: material.m3.clone(),
            normal_secondary: material.normal_secondary.clone(),
            normal_secondary_mask: material.normal_secondary_mask.clone(),
            light_map: material.light_map.clone(),
            ambient_occlusion: material.ambient_occlusion.clone(),
        }
    }
}
impl AsBindGroup for ModelMaterialExtension {
    type Data = ModelMaterialKey;
    type Param = (
        SRes<RenderAssets<GpuImage>>,
        SRes<FallbackImage>,
        SRes<RenderAssets<GpuShaderBuffer>>,
    );
    fn label() -> &'static str {
        "ModelMaterialExtension"
    }
    fn bind_group_data(&self) -> Self::Data {
        ModelMaterialKey::from(self)
    }
    fn unprepared_bind_group(
        &self,
        layout: &BindGroupLayout,
        device: &RenderDevice,
        param: &mut SystemParamItem<'_, '_, Self::Param>,
        force_no_bindless: bool,
    ) -> Result<UnpreparedBindGroup, AsBindGroupError> {
        let mut group = BindingSnapshot::from(self).unprepared_bind_group(
            layout,
            device,
            param,
            force_no_bindless,
        )?;
        if self.m3.settings.x != 0 {
            for (slot, binding) in SAMPLER_BINDINGS.into_iter().enumerate() {
                let Some(image) = self.m3_samplers[slot].as_ref() else {
                    continue;
                };
                let image = param
                    .0
                    .get(image)
                    .ok_or(AsBindGroupError::RetryNextUpdate)?;
                if let Some((_, resource)) = group
                    .bindings
                    .iter_mut()
                    .find(|(index, _)| *index == binding)
                {
                    *resource = OwnedBindingResource::Sampler(
                        SamplerBindingType::Filtering,
                        image.sampler.clone(),
                    );
                }
            }
        }
        Ok(group)
    }
    fn bind_group_layout_entries(
        device: &RenderDevice,
        force_no_bindless: bool,
    ) -> Vec<BindGroupLayoutEntry> {
        let mut entries = BindingSnapshot::bind_group_layout_entries(device, force_no_bindless);
        // Vertex coordinate generation reads only the combiner. Restrict all
        // other resources to avoid redundant per-stage backend bindings.
        for entry in &mut entries {
            entry.visibility = if entry.binding == 100 {
                ShaderStages::VERTEX_FRAGMENT
            } else {
                ShaderStages::FRAGMENT
            };
        }
        debug_assert_eq!(
            entries
                .iter()
                .filter(|entry| matches!(entry.ty, BindingType::Sampler(_)))
                .count(),
            6,
            "the shared material sampler layout must remain within its six extension slots"
        );
        entries
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Assets;
    #[test]
    fn snapshots_preserve_ordinary_image_sampler_pairs_and_keep_pool_ownership_separate() {
        let mut images = Assets::<Image>::default();
        let ordinary = images.add(Image::default());
        let pooled = images.add(Image::default());
        let mut material = ModelMaterialExtension {
            secondary: Some(ordinary.clone()),
            ..Default::default()
        };
        material.m3_samplers[0] = Some(pooled.clone());
        assert_eq!(
            BindingSnapshot::from(&material).secondary,
            Some(ordinary.clone())
        );
        material.m3.settings.x = 1;
        assert_eq!(BindingSnapshot::from(&material).secondary, Some(ordinary));
        assert_eq!(material.m3_samplers[0], Some(pooled));
        assert_eq!(SAMPLER_BINDINGS.len(), 6);
    }
}
