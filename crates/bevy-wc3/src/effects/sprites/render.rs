//! Instanced PRE2 quads evaluated from immutable spawn records.
use crate::effects::blend::EffectBlend;
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
    renderer::{RenderDevice, RenderQueue},
    sync_world::MainEntity,
    texture::{FallbackImage, GpuImage},
    view::{ExtractedView, RenderVisibleEntities},
    Render, RenderApp, RenderStartup, RenderSystems,
};

use super::records::{ParticleEmitterUniform, ParticleInstance, ParticleInstances};
#[cfg(test)]
use crate::effects::records::RecordRing;
use crate::effects::render::{
    effect_mesh, effect_texture_layout, effect_vertex_layout, effect_visible,
    transparent_effect_draw, DrawEffectInstances, EffectBuffer, ExtractedEffectGeometry,
    PassResources, ResidentRecordBuffer, SortPolicy, SortScratch,
};

pub(crate) struct ParticleRenderPlugin;
impl Plugin for ParticleRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<ParticleInstances>::default());
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_render_command::<Transparent3d, DrawParticles>()
            .init_resource::<SpecializedMeshPipelines<ParticlePipeline>>()
            .add_systems(RenderStartup, init_pipeline.after(MeshPipelineSystems))
            .add_systems(
                Render,
                (
                    queue_particles.in_set(RenderSystems::QueueMeshes),
                    prepare_particles.in_set(RenderSystems::PrepareResources),
                ),
            );
    }
}

#[derive(Resource)]
struct ParticlePipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    texture_layout: BindGroupLayoutDescriptor,
    texture_bind_group_layout: BindGroupLayout,
}
fn init_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mesh_pipeline: Res<MeshPipeline>,
    cache: Res<PipelineCache>,
) {
    let texture_layout = effect_texture_layout::<ParticleEmitterUniform, ParticleInstance>(
        "wc3 particle texture layout",
    );
    commands.insert_resource(ParticlePipeline {
        shader: asset_server.load("embedded://bevy_wc3/shaders/wc3_particle.wgsl"),
        mesh_pipeline: mesh_pipeline.clone(),
        texture_bind_group_layout: cache.get_bind_group_layout(&texture_layout),
        texture_layout,
    });
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct ParticleKey {
    mesh: MeshPipelineKey,
    filter: u8,
}
impl SpecializedMeshPipeline for ParticlePipeline {
    type Key = ParticleKey;
    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        if key.filter == 4 {
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
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = self.shader.clone();
            if let Some(Some(target)) = fragment.targets.first_mut() {
                let (src, dst) = match key.filter {
                    1 => (BlendFactor::SrcAlpha, BlendFactor::One),
                    6 => (BlendFactor::One, BlendFactor::One),
                    2 => (BlendFactor::Zero, BlendFactor::Src),
                    3 => (BlendFactor::Dst, BlendFactor::Src),
                    _ => (BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha),
                };
                target.blend = (!matches!(key.filter, 4 | 5)).then_some(BlendState {
                    color: BlendComponent {
                        src_factor: src,
                        dst_factor: dst,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent::OVER,
                });
            }
        }
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(false);
        }
        descriptor.primitive.cull_mode = None;
        Ok(descriptor)
    }
}
fn filter_key(filter: EffectBlend) -> u8 {
    match filter {
        EffectBlend::Alpha => 0,
        EffectBlend::AdditiveAlpha => 1,
        EffectBlend::Modulate => 2,
        EffectBlend::Modulate2x => 3,
        EffectBlend::Cutout => 4,
        EffectBlend::Opaque => 5,
        EffectBlend::Additive => 6,
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_particles(
    draws: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<ParticlePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<ParticlePipeline>>,
    cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    particles: Query<(
        Entity,
        &MainEntity,
        &ParticleInstances,
        &ExtractedEffectGeometry,
    )>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    view_keys: Res<ViewKeyCache>,
) {
    let draw = draws.read().id::<DrawParticles>();
    for (view, visible) in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
            continue;
        };
        let Some(&view_key) = view_keys.get(&view.retained_view_entity) else {
            continue;
        };
        for (entity, main_entity, data, geometry) in &particles {
            if data.live_indices.is_empty() || !effect_visible(visible, *main_entity) {
                continue;
            }
            let Some((mesh, mesh_key)) = effect_mesh(geometry, &meshes, view_key) else {
                continue;
            };
            let Ok(id) = pipelines.specialize(
                &cache,
                &pipeline,
                ParticleKey {
                    mesh: mesh_key,
                    filter: filter_key(data.filter),
                },
                &mesh.layout,
            ) else {
                continue;
            };
            let center = geometry.world_center;
            phase.add_transient(transparent_effect_draw(
                entity,
                *main_entity,
                id,
                draw,
                center,
                data.priority_plane as f32,
            ));
        }
    }
}

type ParticleBuffer = EffectBuffer<u32>;

fn sort_indices(
    data: &ParticleInstances,
    world_from_view: &Affine3A,
    scratch: &mut SortScratch<u32>,
) {
    let eye = world_from_view.translation;
    let forward = -world_from_view.matrix3.z_axis;
    for &slot in data.live_indices.iter() {
        let center = Vec3A::from(data.records[slot as usize].center(&data.uniform));
        scratch.push(slot, (center - eye).dot(forward));
    }
    scratch.sort(false, |slot, order| order.push(slot));
}
#[cfg(test)]
fn sorted_indices(data: &ParticleInstances, world_from_view: &Affine3A) -> Vec<u32> {
    let mut scratch = SortScratch::default();
    sort_indices(data, world_from_view, &mut scratch);
    scratch.order
}

#[allow(clippy::too_many_arguments)]
fn prepare_particles(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &MainEntity,
        &ParticleInstances,
        Option<&mut ParticleBuffer>,
    )>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<ParticlePipeline>,
    images: Res<RenderAssets<GpuImage>>,
    fallback: Res<FallbackImage>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    mut scratch: Local<Vec<ParticleInstance>>,
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
        let records = ResidentRecordBuffer::prepare(
            &data.records,
            previous.as_deref().map(|p| &p.records),
            &device,
            &queue,
            &mut scratch,
        );
        let image = data
            .texture
            .as_ref()
            .and_then(|handle| images.get(handle))
            .unwrap_or(&fallback.d2);
        let mut buffer = ParticleBuffer::prepare(
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
        );
        buffer.prepare_orders(
            views.iter().map(|(view, visible)| {
                (
                    view.retained_view_entity,
                    view.world_from_view.affine(),
                    effect_visible(visible, *main_entity),
                )
            }),
            if data.sort_far {
                SortPolicy::Far
            } else {
                SortPolicy::None
            },
            &mut order_scratch,
            &device,
            &queue,
            |view, scratch| sort_indices(data, view, scratch),
        );
        commands.entity(entity).insert(buffer);
    }
}

type DrawParticles = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    DrawEffectInstances<ParticleBuffer>,
);
#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
