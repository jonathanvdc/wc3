//! Instanced PRE2 quads evaluated from immutable spawn records.
use crate::effects::blend::EffectBlend;
use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::ecs::system::{lifetimeless::*, SystemParamItem};
use bevy::math::{Affine3A, Vec3A};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    self, MeshInputUniform, MeshPipeline, MeshPipelineKey, MeshPipelineSystems, MeshUniform,
    RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup, SetMeshViewBindingArrayBindGroup,
    ViewKeyCache,
};
use bevy::prelude::*;
use bevy::render::{
    batching::gpu_preprocessing::BatchedInstanceBuffers,
    extract_component::ExtractComponentPlugin,
    mesh::{allocator::MeshAllocator, RenderMesh, RenderMeshBufferInfo},
    render_asset::RenderAssets,
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
        SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
    },
    render_resource::*,
    renderer::{RenderDevice, RenderQueue},
    sync_world::MainEntity,
    texture::{FallbackImage, GpuImage},
    view::ExtractedView,
    Render, RenderApp, RenderStartup, RenderSystems,
};

use super::records::{ParticleEmitterUniform, ParticleInstance, ParticleInstances};
#[cfg(test)]
use crate::effects::records::RecordRing;
use crate::effects::render::{
    effect_texture_layout, effect_vertex_layout, transparent_effect_draw, EffectBuffer,
    PassResources, ResidentRecordBuffer,
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
    mesh_instances: Res<RenderMeshInstances>,
    batched: Option<Res<BatchedInstanceBuffers<MeshUniform, MeshInputUniform>>>,
    particles: Query<(Entity, &MainEntity, &ParticleInstances)>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<&ExtractedView>,
    view_keys: Res<ViewKeyCache>,
) {
    let draw = draws.read().id::<DrawParticles>();
    for view in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
            continue;
        };
        let Some(&view_key) = view_keys.get(&view.retained_view_entity) else {
            continue;
        };
        for (entity, main_entity, data) in &particles {
            if data.live_indices.is_empty() {
                continue;
            }
            let Some(mesh_instance) = mesh_instances.render_mesh_queue_data(*main_entity) else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id()) else {
                continue;
            };
            let mesh_key = view_key
                | MeshPipelineKey::from_primitive_topology_and_strip_index(
                    mesh.primitive_topology(),
                    mesh.index_format(),
                );
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
            let center = pbr::get_mesh_instance_world_from_local(
                *main_entity,
                mesh_instance.current_uniform_index,
                &mesh_instances,
                batched.as_deref(),
            )
            .transform_point3(mesh.aabb_center);
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

fn sorted_indices(data: &ParticleInstances, world_from_view: &Affine3A) -> Vec<u32> {
    let eye = world_from_view.translation;
    let forward = -world_from_view.matrix3.z_axis;
    // Evaluate each center once, rather than once per sort comparison.
    let mut depths: Vec<_> = data
        .live_indices
        .iter()
        .map(|&slot| {
            let center = Vec3A::from(data.records[slot as usize].center(&data.uniform));
            (slot, (center - eye).dot(forward))
        })
        .collect();
    depths.sort_by(|a, b| b.1.total_cmp(&a.1));
    depths.into_iter().map(|(slot, _)| slot).collect()
}

#[allow(clippy::too_many_arguments)]
fn prepare_particles(
    mut commands: Commands,
    query: Query<(Entity, &ParticleInstances, Option<&ParticleBuffer>)>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<ParticlePipeline>,
    images: Res<RenderAssets<GpuImage>>,
    fallback: Res<FallbackImage>,
    views: Query<&ExtractedView>,
    mut scratch: Local<Vec<ParticleInstance>>,
) {
    for (entity, data, previous) in &query {
        if data.live_indices.is_empty() {
            // Keep resident records and allocated capacity across emission gaps.
            continue;
        }
        let records = ResidentRecordBuffer::prepare(
            &data.records,
            previous.map(|p| &p.records),
            &device,
            &queue,
            &mut scratch,
        );
        let image = data
            .texture
            .as_ref()
            .and_then(|handle| images.get(handle))
            .unwrap_or(&fallback.d2);
        let sorted_orders = views.iter().filter(|_| data.sort_far).map(|view| {
            (
                view.retained_view_entity,
                sorted_indices(data, &view.world_from_view.affine()),
            )
        });
        let buffer = ParticleBuffer::prepare(
            records,
            &data.live_indices,
            &data.uniform,
            previous,
            PassResources {
                device: &device,
                queue: &queue,
                layout: &pipeline.texture_bind_group_layout,
                image,
            },
            sorted_orders,
        );
        commands.entity(entity).insert(buffer);
    }
}

type DrawParticles = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetMeshBindGroup<2>,
    DrawParticleInstances,
);
struct DrawParticleInstances;
impl<P: PhaseItem> RenderCommand<P> for DrawParticleInstances {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = Read<ExtractedView>;
    type ItemQuery = Read<ParticleBuffer>;
    fn render<'w>(
        item: &P,
        view: &'w ExtractedView,
        buffer: Option<&'w ParticleBuffer>,
        (meshes, instances, allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let allocator = allocator.into_inner();
        let Some(mesh_instance) = instances.render_mesh_queue_data(item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id()) else {
            return RenderCommandResult::Skip;
        };
        let Some(buffer) = buffer else {
            return RenderCommandResult::Skip;
        };
        let Some(vertex) = allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id()) else {
            return RenderCommandResult::Skip;
        };
        pass.set_vertex_buffer(0, vertex.buffer.slice(..));
        let records = buffer
            .sorted
            .get(&view.retained_view_entity)
            .unwrap_or(&buffer.order);
        pass.set_vertex_buffer(1, records.buffer.slice(..));
        pass.set_bind_group(3, &buffer.texture, &[]);
        match &mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count,
            } => {
                let Some(index) = allocator.mesh_index_slice(&mesh_instance.mesh_asset_id()) else {
                    return RenderCommandResult::Skip;
                };
                pass.set_index_buffer(index.buffer.slice(..), *index_format);
                pass.draw_indexed(
                    index.range.start..index.range.start + count,
                    vertex.range.start as i32,
                    0..buffer.live_indices.len() as u32,
                );
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(vertex.range, 0..buffer.live_indices.len() as u32)
            }
        }
        RenderCommandResult::Success
    }
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
