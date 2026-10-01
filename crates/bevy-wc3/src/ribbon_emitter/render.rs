//! GPU ribbon quads built from pairs of immutable world-space cross-sections.
use bevy::core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d};
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::{lifetimeless::*, SystemParamItem};
use bevy::math::{Affine3A, Vec3A};
use bevy::mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout};
use bevy::pbr::{
    MeshPipeline, MeshPipelineKey, MeshPipelineSystems, RenderMeshInstances, SetMeshBindGroup,
    SetMeshViewBindGroup, SetMeshViewBindingArrayBindGroup, ViewKeyCache,
};
use bevy::prelude::*;
use bevy::render::{
    extract_component::{ExtractComponent, ExtractComponentPlugin},
    mesh::{allocator::MeshAllocator, RenderMesh, RenderMeshBufferInfo},
    render_asset::RenderAssets,
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
        RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
    },
    render_resource::{
        binding_types::{
            sampler, storage_buffer_read_only_sized, texture_2d, uniform_buffer_sized,
        },
        *,
    },
    renderer::{RenderDevice, RenderQueue},
    sync_component::SyncComponent,
    sync_world::MainEntity,
    texture::{FallbackImage, GpuImage},
    view::{ExtractedView, NoIndirectDrawing},
    Render, RenderApp, RenderStartup, RenderSystems,
};
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::Arc;
use wc3::model::materials::LayerFilterMode;

use crate::effect_ring::RecordRing;
use crate::effects::render::{EffectBuffer, PassResources, ResidentRecordBuffer};
use crate::effects::simulation::split_time;

/// Birth-time endpoints. The split clock retains precision in long-running scenes.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct RibbonSection {
    pub(crate) above_birth: [f32; 4],
    pub(crate) below_birth: [f32; 4],
}

impl RibbonSection {
    pub(crate) fn new(above: Vec3, below: Vec3, birth: f64) -> Self {
        let [high, low] = split_time(birth);
        Self {
            above_birth: [above.x, above.y, above.z, high],
            below_birth: [below.x, below.y, below.z, low],
        }
    }

    fn center(&self, uniform: &RibbonUniform) -> Vec3 {
        let age = ((uniform.clock_gravity[0] - self.above_birth[3])
            + (uniform.clock_gravity[1] - self.below_birth[3]))
            .max(0.0);
        (Vec3::from_slice(&self.above_birth[..3]) + Vec3::from_slice(&self.below_birth[..3])) * 0.5
            - Vec3::Z * (0.5 * uniform.clock_gravity[2] * age * age)
    }
}

/// Endpoint ring indices, position in the chain, and number of segments in that chain.
/// Draw sorting moves this entire record without changing connectivity or UVs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct RibbonSegment(pub(crate) [u32; 4]);

fn segment_center(
    records: &RecordRing<RibbonSection>,
    segment: RibbonSegment,
    uniform: &RibbonUniform,
) -> Vec3 {
    (records[segment.0[0] as usize].center(uniform)
        + records[segment.0[1] as usize].center(uniform))
        * 0.5
}

#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct RibbonUniform {
    pub(crate) color: [f32; 4],
    pub(crate) clock_gravity: [f32; 4],
    pub(crate) atlas_flags: [u32; 4],
    pub(crate) uv_transform: [[f32; 4]; 4],
}

#[derive(Component, Clone)]
pub(crate) struct RibbonInstances {
    pub(crate) records: Arc<RecordRing<RibbonSection>>,
    pub(crate) live_indices: Arc<Vec<RibbonSegment>>,
    pub(crate) uniform: RibbonUniform,
    pub(crate) texture: Option<Handle<Image>>,
    pub(crate) filter: LayerFilterMode,
    pub(crate) priority_plane: i32,
    pub(crate) emitter: Entity,
    pub(crate) layer_index: usize,
    pub(crate) sort_far: bool,
    pub(crate) sort_near: bool,
    pub(crate) no_depth_test: bool,
    pub(crate) no_depth_set: bool,
    pub(crate) two_sided: bool,
}

impl SyncComponent for RibbonInstances {
    type Target = Self;
}
impl ExtractComponent for RibbonInstances {
    type QueryData = (&'static RibbonInstances, &'static InheritedVisibility);
    type QueryFilter = ();
    type Out = Self;
    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        item.1.get().then(|| item.0.clone())
    }
}

pub(crate) struct RibbonRenderPlugin;
impl Plugin for RibbonRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<RibbonInstances>::default());
        app.add_systems(PostUpdate, configure_ribbon_views);
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_render_command::<Transparent3d, DrawRibbons>()
            .init_resource::<SpecializedMeshPipelines<RibbonPipeline>>()
            .add_systems(RenderStartup, init_pipeline.after(MeshPipelineSystems))
            .add_systems(
                Render,
                (
                    queue_ribbons.in_set(RenderSystems::QueueMeshes),
                    prepare_ribbons.in_set(RenderSystems::PrepareResources),
                ),
            );
    }
}

// The custom draw command issues direct instanced draws. Bevy's GPU
// preprocessing must keep direct mesh bindings for views using this pass.
fn configure_ribbon_views(
    mut commands: Commands,
    cameras: Query<Entity, (With<Camera3d>, Without<NoIndirectDrawing>)>,
) {
    for camera in &cameras {
        commands.entity(camera).insert(NoIndirectDrawing);
    }
}

#[derive(Resource)]
struct RibbonPipeline {
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
        "wc3 ribbon texture layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer_sized(false, NonZeroU64::new(size_of::<RibbonUniform>() as u64)),
                storage_buffer_read_only_sized(
                    false,
                    NonZeroU64::new(size_of::<RibbonSection>() as u64),
                ),
            ),
        ),
    );
    commands.insert_resource(RibbonPipeline {
        shader: asset_server.load("embedded://bevy_wc3/shaders/wc3_ribbon.wgsl"),
        mesh_pipeline: mesh_pipeline.clone(),
        texture_bind_group_layout: cache.get_bind_group_layout(&texture_layout),
        texture_layout,
    });
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct RibbonKey {
    mesh: MeshPipelineKey,
    filter: u8,
    no_depth_test: bool,
    no_depth_set: bool,
    two_sided: bool,
}
impl SpecializedMeshPipeline for RibbonPipeline {
    type Key = RibbonKey;
    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        if key.filter == 1 {
            descriptor
                .fragment
                .as_mut()
                .unwrap()
                .shader_defs
                .push("ALPHA_KEY".into());
        }
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: size_of::<RibbonSegment>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![VertexAttribute {
                format: VertexFormat::Uint32x4,
                offset: 0,
                shader_location: 3,
            }],
        });
        descriptor.layout.push(self.texture_layout.clone());
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = self.shader.clone();
            if let Some(Some(target)) = fragment.targets.first_mut() {
                let (src, dst) = match key.filter {
                    3 | 4 => (BlendFactor::SrcAlpha, BlendFactor::One),
                    5 => (BlendFactor::Zero, BlendFactor::Src),
                    6 => (BlendFactor::Dst, BlendFactor::Src),
                    _ => (BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha),
                };
                target.blend = (key.filter >= 2).then_some(BlendState {
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
            depth.depth_write_enabled = Some(!key.no_depth_set && key.filter <= 1);
            if key.no_depth_test {
                depth.depth_compare = Some(CompareFunction::Always);
            }
        }
        descriptor.primitive.cull_mode = if key.two_sided {
            None
        } else {
            Some(Face::Back)
        };
        Ok(descriptor)
    }
}
fn filter_key(filter: LayerFilterMode) -> u8 {
    match filter {
        LayerFilterMode::None => 0,
        LayerFilterMode::Transparent => 1,
        LayerFilterMode::Blend => 2,
        LayerFilterMode::Additive => 3,
        LayerFilterMode::AddAlpha => 4,
        LayerFilterMode::Modulate => 5,
        LayerFilterMode::Modulate2x => 6,
        LayerFilterMode::Unknown(_) => 2,
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_ribbons(
    draws: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<RibbonPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<RibbonPipeline>>,
    cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    mesh_instances: Res<RenderMeshInstances>,
    ribbons: Query<(Entity, &MainEntity, &RibbonInstances)>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<&ExtractedView>,
    view_keys: Res<ViewKeyCache>,
) {
    let draw = draws.read().id::<DrawRibbons>();
    let mut ordered: Vec<_> = ribbons.iter().collect();
    ordered.sort_by_key(|(_, _, data)| (data.emitter.to_bits(), data.layer_index));
    for view in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
            continue;
        };
        let Some(&view_key) = view_keys.get(&view.retained_view_entity) else {
            continue;
        };
        for &(entity, main_entity, data) in &ordered {
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
                RibbonKey {
                    mesh: mesh_key,
                    filter: filter_key(data.filter),
                    no_depth_test: data.no_depth_test,
                    no_depth_set: data.no_depth_set,
                    two_sided: data.two_sided,
                },
                &mesh.layout,
            ) else {
                continue;
            };
            let first = segment_center(&data.records, data.live_indices[0], &data.uniform);
            let last = segment_center(
                &data.records,
                *data.live_indices.last().unwrap(),
                &data.uniform,
            );
            let center = (first + last) * 0.5;
            // Requeued each frame: empty or hidden emitters must not retain a
            // draw referencing their last nonempty GPU buffer.
            phase.add_transient(Transparent3d {
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

type RibbonBuffer = EffectBuffer<RibbonSegment>;

fn sorted_indices(data: &RibbonInstances, world_from_view: &Affine3A) -> Vec<RibbonSegment> {
    let eye = world_from_view.translation;
    let forward = -world_from_view.matrix3.z_axis;
    // Evaluate each center once, rather than once per sort comparison.
    let mut depths: Vec<_> = data
        .live_indices
        .iter()
        .map(|&slot| {
            let center = Vec3A::from(segment_center(&data.records, slot, &data.uniform));
            (slot, (center - eye).dot(forward))
        })
        .collect();
    depths.sort_by(|a, b| {
        if data.sort_near {
            a.1.total_cmp(&b.1)
        } else {
            b.1.total_cmp(&a.1)
        }
    });
    depths.into_iter().map(|(slot, _)| slot).collect()
}

#[allow(clippy::too_many_arguments)]
fn prepare_ribbons(
    mut commands: Commands,
    query: Query<(Entity, &RibbonInstances, Option<&RibbonBuffer>)>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<RibbonPipeline>,
    images: Res<RenderAssets<GpuImage>>,
    fallback: Res<FallbackImage>,
    views: Query<&ExtractedView>,
    mut scratch: Local<Vec<RibbonSection>>,
) {
    // All material layers of an emitter share the same resident section buffer.
    // Uniforms, texture bindings, and draw ordering remain specific to each pass.
    let mut shared_sections: HashMap<Entity, ResidentRecordBuffer> = HashMap::new();
    for (entity, data, previous) in &query {
        if data.live_indices.is_empty() {
            // Keep resident records and allocated capacity across emission gaps.
            continue;
        }
        let records = if let Some(records) = shared_sections.get(&data.emitter) {
            records.clone()
        } else {
            ResidentRecordBuffer::prepare(
                &data.records,
                previous.map(|p| &p.records),
                &device,
                &queue,
                &mut scratch,
            )
        };
        shared_sections.insert(data.emitter, records.clone());
        let image = data
            .texture
            .as_ref()
            .and_then(|handle| images.get(handle))
            .unwrap_or(&fallback.d2);
        let sorted_orders = views
            .iter()
            .filter(|_| data.sort_far || data.sort_near)
            .map(|view| {
                (
                    view.retained_view_entity,
                    sorted_indices(data, &view.world_from_view.affine()),
                )
            });
        let buffer = RibbonBuffer::prepare(
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

type DrawRibbons = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetMeshBindGroup<2>,
    DrawRibbonInstances,
);
struct DrawRibbonInstances;
impl<P: PhaseItem> RenderCommand<P> for DrawRibbonInstances {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = Read<ExtractedView>;
    type ItemQuery = Read<RibbonBuffer>;
    fn render<'w>(
        item: &P,
        view: &'w ExtractedView,
        buffer: Option<&'w RibbonBuffer>,
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
