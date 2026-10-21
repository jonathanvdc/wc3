//! GPU binding layout independent of semantic image/sampler ownership.

use super::{CombinerUniform, HdUniform, ModelMaterialExtension, ModelMaterialKey, WorldUniform};
use bevy::ecs::system::{
    lifetimeless::{SRes, SResMut},
    SystemParamItem,
};
use bevy::prelude::{App, Handle, Image, Resource};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::encase::{internal::WriteInto, UniformBuffer};
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupError, BindGroupLayout, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferInitDescriptor, BufferUsages, OwnedBindingResource, ShaderStages,
    ShaderType, UnpreparedBindGroup,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::GpuShaderBuffer;
use bevy::render::texture::{FallbackImage, GpuImage};
use bevy::render::{GpuResourceAppExt, RenderApp};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[derive(AsBindGroup)]
struct ImageBindings {
    #[texture(101)]
    #[sampler(102)]
    secondary: Option<Handle<Image>>,
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
}
impl From<&ModelMaterialExtension> for ImageBindings {
    fn from(material: &ModelMaterialExtension) -> Self {
        Self {
            secondary: material.secondary.clone(),
            orm: material.orm.clone(),
            team_color: material.team_color.clone(),
            environment: material.environment.clone(),
            tertiary: material.tertiary.clone(),
            fourth: material.fourth.clone(),
        }
    }
}
const UNIFORMS_PER_TYPE: usize = 256;

/// Immutable resources retained for one render-device generation.
/// Public only because it occurs in AsBindGroup's associated system parameter.
#[derive(Resource, Default)]
pub struct MaterialUniformBuffers {
    combiner: UniformBuffers,
    hd: UniformBuffers,
    world: UniformBuffers,
}
#[derive(Default)]
struct UniformBuffers {
    entries: HashMap<Arc<[u8]>, Buffer>,
    insertion_order: VecDeque<Arc<[u8]>>,
}
impl UniformBuffers {
    fn acquire(&mut self, bytes: Vec<u8>, device: &RenderDevice) -> Buffer {
        if let Some(buffer) = self.entries.get(bytes.as_slice()) {
            return buffer.clone();
        }
        let buffer = create_uniform(&bytes, device);
        if self.entries.len() == UNIFORMS_PER_TYPE {
            let oldest = self
                .insertion_order
                .pop_front()
                .expect("full uniform cache");
            self.entries.remove(&oldest);
        }
        let key: Arc<[u8]> = bytes.into();
        self.insertion_order.push_back(key.clone());
        self.entries.insert(key, buffer.clone());
        buffer
    }
}
fn encode_uniform<T: ShaderType + WriteInto>(value: &T) -> Vec<u8> {
    let mut bytes = UniformBuffer::new(Vec::new());
    bytes
        .write(value)
        .expect("material uniform has a valid static layout");
    bytes.into_inner()
}
fn create_uniform(bytes: &[u8], device: &RenderDevice) -> Buffer {
    device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("wc3 immutable material parameters"),
        contents: bytes,
        usage: BufferUsages::UNIFORM,
    })
}
fn uniform_binding<T: ShaderType>(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: if binding == 100 {
            ShaderStages::VERTEX_FRAGMENT
        } else {
            ShaderStages::FRAGMENT
        },
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(T::min_size()),
        },
        count: None,
    }
}
/// Reset cached resources whenever Bevy starts or recovers its render device.
pub(crate) fn register_material_uniform_buffers(app: &mut App) {
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.init_gpu_resource::<MaterialUniformBuffers>();
    }
}
impl AsBindGroup for ModelMaterialExtension {
    type Data = ModelMaterialKey;
    type Param = (
        (
            SRes<RenderAssets<GpuImage>>,
            SRes<FallbackImage>,
            SRes<RenderAssets<GpuShaderBuffer>>,
        ),
        Option<SResMut<MaterialUniformBuffers>>,
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
        let (images, cache) = param;
        // Preserve Bevy's image fallback/retry semantics before allocating uniforms.
        let mut group = ImageBindings::from(self).unprepared_bind_group(
            layout,
            device,
            images,
            force_no_bindless,
        )?;
        let bytes = [
            encode_uniform(&self.combiner),
            encode_uniform(&self.hd),
            encode_uniform(&self.world),
        ];
        let buffers = if let Some(cache) = cache {
            let [combiner, hd, world] = bytes;
            [
                cache.combiner.acquire(combiner, device),
                cache.hd.acquire(hd, device),
                cache.world.acquire(world, device),
            ]
        } else {
            // Standalone MaterialPlugin use remains valid without Wc3BevyPlugin.
            bytes.map(|bytes| create_uniform(&bytes, device))
        };
        group.bindings.extend(
            [100, 103, 110]
                .into_iter()
                .zip(buffers)
                .map(|(binding, buffer)| (binding, OwnedBindingResource::Buffer(buffer))),
        );
        Ok(group)
    }
    fn bind_group_layout_entries(
        device: &RenderDevice,
        force_no_bindless: bool,
    ) -> Vec<BindGroupLayoutEntry> {
        let mut entries = ImageBindings::bind_group_layout_entries(device, force_no_bindless);
        entries.extend([
            uniform_binding::<CombinerUniform>(100),
            uniform_binding::<HdUniform>(103),
            uniform_binding::<WorldUniform>(110),
        ]);
        entries.sort_by_key(|entry| entry.binding);
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
#[path = "bindings_tests.rs"]
mod tests;
