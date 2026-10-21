//! View-specific M2 sprite ordering and GPU corner expansion.
use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::math::{Affine3A, Vec3A};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    MeshPipeline, MeshPipelineKey, MeshPipelineSystems, SetMeshViewBindGroup,
    SetMeshViewBindingArrayBindGroup, ViewKeyCache,
};
use bevy::prelude::*;
use bevy::render::{
    extract_component::ExtractComponentPlugin,
    mesh::RenderMesh,
    render_asset::RenderAssets,
    render_phase::{AddRenderCommand, DrawFunctions, SetItemPipeline, ViewSortedRenderPhases},
    render_resource::*,
    renderer::{RenderAdapter, RenderDevice, RenderQueue},
    sync_world::MainEntity,
    texture::{FallbackImage, GpuImage},
    view::{ExtractedView, RenderVisibleEntities},
    Render, RenderApp, RenderStartup, RenderSystems,
};
#[cfg(test)]
use std::sync::Arc;
use wc3::model::values::model::ParticleBlend;

use super::records::{AgeSpriteComposition, AgeSpriteData, AgeSpriteRecord, AgeSpriteUniform};
use super::refraction::{mask_view_layout, RefractionPhases, RefractionPlugin, SetRefractionView};
use crate::effects::render::{
    effect_mesh, effect_texture_layout, effect_vertex_layout, effect_visible,
    transparent_effect_draw, DrawEffectInstances, EffectBuffer, ExtractedEffectGeometry,
    PassResources, ResidentRecordBuffer, SortPolicy, SortScratch,
};

pub(crate) struct AgeSpriteRenderPlugin;
impl Plugin for AgeSpriteRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<AgeSpriteData>::default(),
            RefractionPlugin,
        ));
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_render_command::<Transparent3d, DrawAgeSprites>()
            .add_render_command::<Transparent3d, DrawRefractionSprites>()
            .init_resource::<SpecializedMeshPipelines<AgeSpritePipeline>>()
            .add_systems(RenderStartup, init_pipeline.after(MeshPipelineSystems))
            .add_systems(
                Render,
                (
                    queue_age_sprites.in_set(RenderSystems::QueueMeshes),
                    prepare_age_sprites.in_set(RenderSystems::PrepareResources),
                ),
            );
    }
}

#[derive(Resource)]
struct AgeSpritePipeline {
    mask_samples: TextureFormatFeatureFlags,
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    texture_layout: BindGroupLayoutDescriptor,
    texture_bind_group_layout: BindGroupLayout,
}
fn init_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    adapter: Res<RenderAdapter>,
    mesh_pipeline: Res<MeshPipeline>,
    cache: Res<PipelineCache>,
) {
    let mut texture_layout =
        effect_texture_layout::<AgeSpriteUniform, AgeSpriteRecord>("wc3 age sprite texture layout");
    use bevy::render::render_resource::binding_types::{sampler, texture_2d};
    texture_layout.entries.extend(
        BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (4, texture_2d(TextureSampleType::Float { filterable: true })),
                (5, sampler(SamplerBindingType::Filtering)),
                (6, texture_2d(TextureSampleType::Float { filterable: true })),
                (7, sampler(SamplerBindingType::Filtering)),
            ),
        )
        .to_vec(),
    );
    commands.insert_resource(AgeSpritePipeline {
        mask_samples: adapter
            .get_texture_format_features(TextureFormat::Rgba16Float)
            .flags,
        shader: asset_server.load("embedded://bevy_wc3/shaders/wc3_age_particle.wgsl"),
        mesh_pipeline: mesh_pipeline.clone(),
        texture_bind_group_layout: cache.get_bind_group_layout(&texture_layout),
        texture_layout,
    });
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct AgeSpriteKey {
    mesh: MeshPipelineKey,
    filter: u8,
    refraction: bool,
    sampled: bool,
    ribbon: bool,
}
impl SpecializedMeshPipeline for AgeSpritePipeline {
    type Key = AgeSpriteKey;
    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        if key.sampled {
            descriptor.vertex.shader_defs.push("SAMPLED_SPRITE".into());
        }
        if key.ribbon {
            descriptor.vertex.shader_defs.push("SAMPLED_RIBBON".into());
        }
        if key.filter == 1 {
            descriptor
                .fragment
                .as_mut()
                .unwrap()
                .shader_defs
                .push("ALPHA_KEY".into());
        }
        descriptor
            .vertex
            .buffers
            .push(effect_vertex_layout::<u32>(VertexFormat::Uint32));
        // The shader reads emitter-owned transforms and birth records, so it
        // needs only Bevy's view groups, not a preprocessed mesh bind group.
        descriptor.layout.truncate(2);
        descriptor.layout.push(self.texture_layout.clone());
        if key.refraction {
            descriptor.layout.push(mask_view_layout());
        }
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = self.shader.clone();
            if key.sampled {
                fragment.shader_defs.push("SAMPLED_SPRITE".into());
            }
            if key.refraction {
                fragment.shader_defs.push("REFRACTION".into());
            }
            if let Some(Some(target)) = fragment.targets.first_mut() {
                target.blend = if key.refraction {
                    Some(mask_blend_state(key.filter))
                } else {
                    blend_state(key.filter)
                };
                if key.refraction {
                    target.format = TextureFormat::Rgba16Float;
                }
            }
        }
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(!key.refraction && matches!(key.filter, 0 | 1));
        }
        descriptor.primitive.cull_mode = None;
        Ok(descriptor)
    }
}
fn blend_state(filter: u8) -> Option<BlendState> {
    if matches!(filter, 0 | 1) {
        return None;
    }
    let (src_factor, dst_factor) = match filter {
        3 => (BlendFactor::One, BlendFactor::One),
        4 => (BlendFactor::SrcAlpha, BlendFactor::One),
        5 => (BlendFactor::Zero, BlendFactor::Src),
        6 => (BlendFactor::Dst, BlendFactor::Src),
        7 => (BlendFactor::One, BlendFactor::OneMinusSrcAlpha),
        _ => (BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha),
    };
    Some(BlendState {
        color: BlendComponent {
            src_factor,
            dst_factor,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent::OVER,
    })
}

// Mask accumulation is its own material interpretation: opaque/cutout inputs
// fade into the mask, and every alpha channel follows the same operation as red.
fn mask_blend_state(filter: u8) -> BlendState {
    let (color, alpha) = match filter {
        3 | 4 => (
            (BlendFactor::SrcAlpha, BlendFactor::One),
            (BlendFactor::SrcAlpha, BlendFactor::One),
        ),
        5 => (
            (BlendFactor::Zero, BlendFactor::Src),
            (BlendFactor::Zero, BlendFactor::SrcAlpha),
        ),
        6 => (
            (BlendFactor::Dst, BlendFactor::Src),
            (BlendFactor::DstAlpha, BlendFactor::SrcAlpha),
        ),
        _ => (
            (BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha),
            (BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha),
        ),
    };
    let component = |(src_factor, dst_factor)| BlendComponent {
        src_factor,
        dst_factor,
        operation: BlendOperation::Add,
    };
    BlendState {
        color: component(color),
        alpha: component(alpha),
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_age_sprites(
    draws: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<AgeSpritePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<AgeSpritePipeline>>,
    cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    particles: Query<(
        Entity,
        &MainEntity,
        &AgeSpriteData,
        &ExtractedEffectGeometry,
    )>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    mut refraction: ResMut<RefractionPhases>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    view_keys: Res<ViewKeyCache>,
) {
    let draws = draws.read();
    let draw = draws.id::<DrawAgeSprites>();
    let mask_draw = draws.id::<DrawRefractionSprites>();
    for (view, visible) in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
            continue;
        };
        let Some(&view_key) = view_keys.get(&view.retained_view_entity) else {
            continue;
        };
        refraction
            .0
            .prepare_for_new_frame(view.retained_view_entity);
        let mask_phase = refraction.0.get_mut(&view.retained_view_entity).unwrap();
        for (entity, main_entity, data, geometry) in &particles {
            if data.live_indices.is_empty() || !effect_visible(visible, *main_entity) {
                continue;
            }
            let Some((mesh, mesh_key)) = effect_mesh(geometry, &meshes, view_key) else {
                continue;
            };
            if data.style.composition == AgeSpriteComposition::Refraction
                && (!pipeline
                    .mask_samples
                    .sample_count_supported(mesh_key.msaa_samples())
                    || (mesh_key.msaa_samples() > 1
                        && !pipeline
                            .mask_samples
                            .contains(TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)))
            {
                warn_once!("Particle refraction mask does not support {}x MSAA on this adapter; distortion omitted", mesh_key.msaa_samples());
                continue;
            }
            let Ok(id) = pipelines.specialize(
                &cache,
                &pipeline,
                AgeSpriteKey {
                    mesh: mesh_key,
                    refraction: data.style.composition == AgeSpriteComposition::Refraction,
                    sampled: matches!(
                        data.style.composition,
                        AgeSpriteComposition::Sampled { .. } | AgeSpriteComposition::Ribbon { .. }
                    ),
                    ribbon: matches!(data.style.composition, AgeSpriteComposition::Ribbon { .. }),
                    filter: match data.style.blend {
                        ParticleBlend::Unknown(_) => 2,
                        blend => blend.raw() as u8,
                    },
                },
                &mesh.layout,
            ) else {
                continue;
            };
            let center = geometry.world_center;
            let is_mask = data.style.composition == AgeSpriteComposition::Refraction;
            let destination = if is_mask {
                &mut *mask_phase
            } else {
                &mut *phase
            };
            destination.add_transient(transparent_effect_draw(
                entity,
                *main_entity,
                id,
                if is_mask { mask_draw } else { draw },
                center,
                data.style.priority,
            ));
        }
    }
}

type AgeSpriteBuffer = EffectBuffer<u32>;

fn sort_indices(data: &AgeSpriteData, world_from_view: &Affine3A, scratch: &mut SortScratch<u32>) {
    let eye = world_from_view.translation;
    let forward = -world_from_view.matrix3.z_axis;
    for (index, record) in data.records.iter().enumerate() {
        let center = if matches!(data.style.composition, AgeSpriteComposition::Ribbon { .. }) {
            let parameters = record.ribbon_parameters();
            Vec3A::from(parameters.start * 0.5 + parameters.end * 0.5)
        } else {
            Vec3A::from(Vec3::from_slice(&record.position[..3]))
        };
        scratch.push(index as u32, (center - eye).dot(forward));
    }
    scratch.sort(data.style.sort == SortPolicy::Near, |index, order| {
        if data.style.head {
            order.push(index);
        }
        if data.style.tail {
            order.push(index | 0x8000_0000);
        }
    });
}
#[cfg(test)]
pub(crate) fn sort_for_benchmark(
    data: &AgeSpriteData,
    world_from_view: &Affine3A,
    scratch: &mut SortScratch<u32>,
) {
    sort_indices(data, world_from_view, scratch);
}
#[cfg(test)]
fn sorted_indices(data: &AgeSpriteData, world_from_view: &Affine3A) -> Vec<u32> {
    let mut scratch = SortScratch::default();
    sort_indices(data, world_from_view, &mut scratch);
    scratch.order
}

#[allow(clippy::too_many_arguments)]
fn prepare_age_sprites(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &MainEntity,
        &AgeSpriteData,
        Option<&mut AgeSpriteBuffer>,
    )>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<AgeSpritePipeline>,
    images: Res<RenderAssets<GpuImage>>,
    fallback: Res<FallbackImage>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    mut order_scratch: Local<SortScratch<u32>>,
) {
    for (entity, main_entity, data, mut previous) in &mut query {
        if let Some(previous) = previous.as_deref_mut() {
            previous.retain_order_views(|key| {
                views
                    .iter()
                    .any(|(view, _)| view.retained_view_entity == key)
            });
        }
        if data.live_indices.is_empty()
            || !views
                .iter()
                .any(|(_, visible)| effect_visible(visible, *main_entity))
        {
            if let Some(previous) = previous.as_deref_mut() {
                previous.suspend_draws();
            }
            // Keep resident records and allocated capacity across emission gaps.
            continue;
        }
        let records = ResidentRecordBuffer::prepare_slice(
            &data.records,
            previous.as_deref().map(|p| &p.records),
            &device,
            &queue,
        );
        let textures = data.style.textures.each_ref().map(|handle| {
            handle
                .as_ref()
                .and_then(|handle| images.get(handle))
                .unwrap_or(&fallback.d2)
        });
        let [image, secondary, tertiary] = textures;
        let mut buffer = AgeSpriteBuffer::prepare_layers(
            records,
            &data.live_indices,
            &data.uniform,
            previous.as_deref_mut(),
            PassResources {
                device: &device,
                queue: &queue,
                layout: &pipeline.texture_bind_group_layout,
                image,
            },
            Some([secondary, tertiary]),
        );
        buffer.prepare_orders(
            views.iter().map(|(view, visible)| {
                (
                    view.retained_view_entity,
                    view.world_from_view.affine(),
                    effect_visible(visible, *main_entity),
                )
            }),
            data.style.sort,
            &mut order_scratch,
            &device,
            &queue,
            |view, scratch| sort_indices(data, view, scratch),
        );
        commands.entity(entity).insert(buffer);
    }
}

type DrawAgeSprites = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    DrawEffectInstances<AgeSpriteBuffer>,
);
type DrawRefractionSprites = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetRefractionView,
    DrawEffectInstances<AgeSpriteBuffer>,
);
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
