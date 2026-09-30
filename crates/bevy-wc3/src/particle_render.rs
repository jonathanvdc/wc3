//! Instanced PRE2 quads rendered from compact per-particle records.
use bevy::core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d};
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::{lifetimeless::*, SystemParamItem};
use bevy::mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout};
use bevy::pbr::{
    self, MeshInputUniform, MeshPipeline, MeshPipelineKey, MeshPipelineSystems, MeshUniform,
    RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup, SetMeshViewBindingArrayBindGroup,
    ViewKeyCache,
};
use bevy::prelude::*;
use bevy::render::{
    batching::gpu_preprocessing::BatchedInstanceBuffers,
    extract_component::{ExtractComponent, ExtractComponentPlugin},
    mesh::{allocator::MeshAllocator, RenderMesh, RenderMeshBufferInfo},
    render_asset::RenderAssets,
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
        RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
    },
    render_resource::{
        binding_types::{sampler, texture_2d},
        *,
    },
    renderer::{RenderDevice, RenderQueue},
    sync_component::SyncComponent,
    sync_world::MainEntity,
    texture::{FallbackImage, GpuImage},
    view::{ExtractedView, NoIndirectDrawing, RetainedViewEntity},
    Render, RenderApp, RenderStartup, RenderSystems,
};
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use wc3::model::emitters::Particle2FilterMode;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct ParticleInstance {
    pub(crate) center_size: [f32; 4],
    pub(crate) velocity_tail: [f32; 4],
    pub(crate) color: [f32; 4],
    pub(crate) uv_rect: [f32; 4],
    pub(crate) flags: [f32; 4],
}

#[derive(Component, Clone)]
pub(crate) struct ParticleInstances {
    pub(crate) particles: Vec<ParticleInstance>,
    pub(crate) texture: Option<Handle<Image>>,
    pub(crate) filter: Particle2FilterMode,
    pub(crate) priority_plane: u32,
    pub(crate) sort_far: bool,
}

impl SyncComponent for ParticleInstances {
    type Target = Self;
}
impl ExtractComponent for ParticleInstances {
    type QueryData = &'static ParticleInstances;
    type QueryFilter = ();
    type Out = Self;
    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

pub(crate) struct ParticleRenderPlugin;
impl Plugin for ParticleRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<ParticleInstances>::default());
        app.add_systems(PostUpdate, configure_particle_views);
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

// The custom draw command issues direct instanced draws. Bevy's GPU
// preprocessing must keep direct mesh bindings for views using this pass.
fn configure_particle_views(
    mut commands: Commands,
    cameras: Query<Entity, (With<Camera3d>, Without<NoIndirectDrawing>)>,
) {
    for camera in &cameras {
        commands.entity(camera).insert(NoIndirectDrawing);
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
    let texture_layout = BindGroupLayoutDescriptor::new(
        "wc3 particle texture layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
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
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: size_of::<ParticleInstance>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: (0..5)
                .map(|i| VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: i * 16,
                    shader_location: 3 + i as u32,
                })
                .collect(),
        });
        descriptor.layout.push(self.texture_layout.clone());
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = self.shader.clone();
            if let Some(Some(target)) = fragment.targets.first_mut() {
                let (src, dst) = match key.filter {
                    1 => (BlendFactor::SrcAlpha, BlendFactor::One),
                    2 => (BlendFactor::Zero, BlendFactor::Src),
                    3 => (BlendFactor::Dst, BlendFactor::Src),
                    _ => (BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha),
                };
                target.blend = (key.filter != 4).then_some(BlendState {
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
fn filter_key(filter: Particle2FilterMode) -> u8 {
    match filter {
        Particle2FilterMode::Additive => 1,
        Particle2FilterMode::Modulate => 2,
        Particle2FilterMode::Modulate2x => 3,
        Particle2FilterMode::AlphaKey => 4,
        _ => 0,
    }
}

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
            if data.particles.is_empty() {
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
            phase.add_retained(Transparent3d {
                sorting_info: TransparentSortingInfo3d::Sorted {
                    mesh_center: center,
                    depth_bias: data.priority_plane as f32,
                },
                entity: (entity, *main_entity),
                pipeline: id,
                draw_function: draw,
                distance: 0.0,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}

#[derive(Component)]
struct ParticleBuffer {
    buffer: Buffer,
    length: usize,
    capacity: usize,
    texture: BindGroup,
    sorted: HashMap<RetainedViewEntity, Buffer>,
}
fn prepare_particles(
    mut commands: Commands,
    query: Query<(Entity, &ParticleInstances, Option<&ParticleBuffer>)>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<ParticlePipeline>,
    images: Res<RenderAssets<GpuImage>>,
    fallback: Res<FallbackImage>,
    views: Query<&ExtractedView>,
) {
    for (entity, data, previous) in &query {
        if data.particles.is_empty() {
            commands.entity(entity).remove::<ParticleBuffer>();
            continue;
        }
        let bytes = bytemuck::cast_slice(&data.particles);
        let (buffer, capacity) = if let Some(previous) =
            previous.filter(|previous| previous.capacity >= data.particles.len())
        {
            queue.write_buffer(&previous.buffer, 0, bytes);
            (previous.buffer.clone(), previous.capacity)
        } else {
            (
                device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("wc3 particle records"),
                    contents: bytes,
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                }),
                data.particles.len(),
            )
        };
        let image = data
            .texture
            .as_ref()
            .and_then(|handle| images.get(handle))
            .unwrap_or(&fallback.d2);
        let texture = device.create_bind_group(
            "wc3 particle texture",
            &pipeline.texture_bind_group_layout,
            &BindGroupEntries::sequential((&image.texture_view, &image.sampler)),
        );
        let mut sorted = HashMap::new();
        if data.sort_far {
            for view in &views {
                let eye = view.world_from_view.translation();
                let mut records = data.particles.clone();
                records.sort_by(|a, b| {
                    let a =
                        Vec3::from_array([a.center_size[0], a.center_size[1], a.center_size[2]])
                            .distance_squared(eye);
                    let b =
                        Vec3::from_array([b.center_size[0], b.center_size[1], b.center_size[2]])
                            .distance_squared(eye);
                    b.total_cmp(&a)
                });
                sorted.insert(
                    view.retained_view_entity,
                    device.create_buffer_with_data(&BufferInitDescriptor {
                        label: Some("wc3 sorted particle records"),
                        contents: bytemuck::cast_slice(&records),
                        usage: BufferUsages::VERTEX,
                    }),
                );
            }
        }
        commands.entity(entity).insert(ParticleBuffer {
            buffer,
            length: data.particles.len(),
            capacity,

            texture,
            sorted,
        });
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
            .unwrap_or(&buffer.buffer);
        pass.set_vertex_buffer(1, records.slice(..));
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
                    0..buffer.length as u32,
                );
            }
            RenderMeshBufferInfo::NonIndexed => pass.draw(vertex.range, 0..buffer.length as u32),
        }
        RenderCommandResult::Success
    }
}
