//! Instanced PRE2 quads evaluated from immutable spawn records.
use bevy::core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d};
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::{lifetimeless::*, SystemParamItem};
use bevy::math::{Affine3A, Vec3A};
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
use std::array::from_fn;
use std::f32::consts::{FRAC_PI_8, PI};
use std::num::NonZeroU64;
use std::sync::Arc;
use wc3::model::emitters::{Particle2FilterMode, ParticleEmitter2};

use crate::effect_ring::RecordRing;
use crate::effects::render::{EffectBuffer, PassResources, ResidentRecordBuffer};
use crate::effects::simulation::split_time;

/// Immutable spawn data indexed by the growing record ring.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct ParticleInstance {
    position_birth: [f32; 4],
    velocity_gravity: [f32; 4],
    // Lifetime, tail marker, low birth time, padding.
    lifetime_tail_birth: [f32; 4],
    // XYZ size multipliers sampled at birth, then XYQuad facing in radians.
    scale_facing: [f32; 4],
}

impl ParticleInstance {
    pub(crate) fn new(
        position: Vec3,
        velocity: Vec3,
        gravity: f32,
        birth_time: f64,
        lifetime: f32,
        size_scale: Vec3,
        tail: bool,
    ) -> Self {
        let [high, low] = split_time(birth_time);
        Self {
            position_birth: [position.x, position.y, position.z, high],
            velocity_gravity: [velocity.x, velocity.y, velocity.z, gravity],
            lifetime_tail_birth: [lifetime, f32::from(tail), low, 0.0],
            // XYQuad rotates its texture axes by the initial XY velocity angle
            // minus pi plus pi/8. Store it independently of later velocity changes.
            // atan2 also yields a finite facing for vertical/zero speed.
            scale_facing: [
                size_scale.x,
                size_scale.y,
                size_scale.z,
                velocity.y.atan2(velocity.x) - PI + FRAC_PI_8,
            ],
        }
    }

    pub(crate) fn as_tail(mut self) -> Self {
        self.lifetime_tail_birth[1] = 1.0;
        self
    }

    #[cfg(test)]
    pub(crate) fn spawn_scale(&self) -> Vec3 {
        Vec3::from_slice(&self.scale_facing[..3])
    }

    #[cfg(test)]
    pub(crate) fn is_tail(&self) -> bool {
        self.lifetime_tail_birth[1] > 0.5
    }

    pub(crate) fn center(&self, uniform: &ParticleEmitterUniform) -> Vec3 {
        let age = ((uniform.clock_tail[0] - self.position_birth[3])
            + (uniform.clock_tail[1] - self.lifetime_tail_birth[2]))
            .max(0.0);
        let position = Vec3::from_slice(&self.position_birth[..3]);
        let velocity = Vec3::from_slice(&self.velocity_gravity[..3]);
        let center =
            position + velocity * age - Vec3::Z * (0.5 * self.velocity_gravity[3] * age * age);
        if uniform.atlas_flags[2] != 0 {
            Mat4::from_cols_array_2d(&uniform.world_from_local).transform_point3(center)
        } else {
            center
        }
    }
}

/// All fields occupy whole 16-byte lanes, matching the WGSL uniform layout.
#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct ParticleEmitterUniform {
    world_from_local: [[f32; 4]; 4],
    colors: [[f32; 4]; 3],
    scaling: [f32; 4],
    intervals: [[u32; 4]; 4],
    atlas_flags: [u32; 4],
    clock_tail: [f32; 4],
    // Unshaded, followed by reserved lanes.
    render_flags: [u32; 4],
}

impl ParticleEmitterUniform {
    pub(crate) fn new(
        definition: &ParticleEmitter2,
        transform: &GlobalTransform,
        time: f64,
    ) -> Self {
        let [high, low] = split_time(time);
        Self {
            world_from_local: transform.to_matrix().to_cols_array_2d(),
            colors: from_fn(|i| {
                let color = definition.segment_colors[i];
                [
                    color[0],
                    color[1],
                    color[2],
                    definition.alpha[i] as f32 / 255.0,
                ]
            }),
            scaling: [
                definition.particle_scaling[0],
                definition.particle_scaling[1],
                definition.particle_scaling[2],
                definition.time.clamp(0.001, 0.999),
            ],
            intervals: definition
                .uv_animations
                .map(|interval| [interval[0], interval[1], interval[2], 0]),
            atlas_flags: [
                definition.rows.max(1),
                definition.columns.max(1),
                u32::from(definition.node.flags.model_space()),
                u32::from(definition.node.flags.xy_quad()),
            ],
            clock_tail: [high, low, definition.tail_length, 0.0],
            render_flags: [u32::from(definition.node.flags.unshaded()), 0, 0, 0],
        }
    }
}

#[derive(Component, Clone)]
pub(crate) struct ParticleInstances {
    pub(crate) records: Arc<RecordRing<ParticleInstance>>,
    pub(crate) live_indices: Arc<Vec<u32>>,
    pub(crate) uniform: ParticleEmitterUniform,
    pub(crate) texture: Option<Handle<Image>>,
    pub(crate) filter: Particle2FilterMode,
    pub(crate) priority_plane: u32,
    pub(crate) sort_far: bool,
}

impl SyncComponent for ParticleInstances {
    type Target = Self;
}
impl ExtractComponent for ParticleInstances {
    type QueryData = (&'static ParticleInstances, &'static InheritedVisibility);
    type QueryFilter = ();
    type Out = Self;
    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        item.1.get().then(|| item.0.clone())
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
            ShaderStages::VERTEX_FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer_sized(
                    false,
                    NonZeroU64::new(size_of::<ParticleEmitterUniform>() as u64),
                ),
                storage_buffer_read_only_sized(
                    false,
                    NonZeroU64::new(size_of::<ParticleInstance>() as u64),
                ),
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
            array_stride: size_of::<u32>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![VertexAttribute {
                format: VertexFormat::Uint32,
                offset: 0,
                shader_location: 3,
            }],
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
mod tests {
    use super::*;
    use naga::front::wgsl::parse_str;
    use naga::valid::{Capabilities, ValidationFlags, Validator};
    use naga::TypeInner;
    use wc3::model::scene::Node;

    fn emitter() -> ParticleEmitter2 {
        ParticleEmitter2::new(Node::new("Particles", 0).unwrap())
    }

    fn instances(
        records: Vec<ParticleInstance>,
        uniform: ParticleEmitterUniform,
    ) -> ParticleInstances {
        ParticleInstances {
            live_indices: Arc::new((0..records.len() as u32).collect()),
            records: Arc::new(records.into()),
            uniform,
            texture: None,
            filter: Particle2FilterMode::Blend,
            priority_plane: 0,
            sort_far: true,
        }
    }

    #[test]
    fn hidden_emitters_are_removed_from_render_extraction_and_can_reappear() {
        let data = instances(Vec::new(), ParticleEmitterUniform::default());
        assert!(
            ParticleInstances::extract_component((&data, &InheritedVisibility::VISIBLE)).is_some()
        );
        assert!(
            ParticleInstances::extract_component((&data, &InheritedVisibility::HIDDEN)).is_none()
        );
        assert!(
            ParticleInstances::extract_component((&data, &InheritedVisibility::VISIBLE)).is_some()
        );
    }

    #[test]
    fn wrapped_particle_indices_sort_by_depth_instead_of_physical_position() {
        let mut records = RecordRing::default();
        let record = |z: f32| {
            ParticleInstance::new(Vec3::Z * z, Vec3::ZERO, 0.0, 0.0, 1.0, Vec3::ONE, false)
        };
        for z in [0.0, 1.0, 2.0, 3.0] {
            records.push(record(z), 4);
        }
        records.pop_front();
        records.pop_front();
        records.push(record(4.0), 4);
        records.push(record(5.0), 4);
        let mut data = instances(Vec::new(), ParticleEmitterUniform::default());
        data.live_indices = Arc::new((0..records.len()).map(|i| records.index(i)).collect());
        data.records = Arc::new(records);
        assert_eq!(*data.live_indices, vec![2, 3, 0, 1]);
        assert_eq!(sorted_indices(&data, &Affine3A::IDENTITY), vec![2, 3, 0, 1]);
        let reverse = Affine3A::from_quat(Quat::from_rotation_y(PI));
        assert_eq!(sorted_indices(&data, &reverse), vec![1, 0, 3, 2]);
    }

    #[test]
    fn xy_facing_survives_gravity_and_tail_conversion() {
        let record = ParticleInstance::new(
            Vec3::ZERO,
            Vec3::new(0.0, 2.0, 3.0),
            20.0,
            0.0,
            4.0,
            Vec3::new(2.0, 0.5, 3.0),
            false,
        );
        let facing = record.scale_facing[3];
        assert!((facing - (-PI * 0.5 + FRAC_PI_8)).abs() < 0.00001);
        let uniform = ParticleEmitterUniform::new(&emitter(), &GlobalTransform::default(), 1.0);
        assert!(record.center(&uniform).z < 0.0);
        assert_eq!(record.as_tail().scale_facing, record.scale_facing);
        for velocity in [Vec3::ZERO, Vec3::Z, -Vec3::Z] {
            let stationary =
                ParticleInstance::new(Vec3::ZERO, velocity, 0.0, 0.0, 4.0, Vec3::ONE, false);
            assert!(stationary.scale_facing[3].is_finite());
        }
    }

    #[test]
    fn unshaded_is_a_render_flag_independent_of_clock_and_tail_length() {
        let mut definition = emitter();
        let shaded = ParticleEmitterUniform::new(&definition, &GlobalTransform::default(), 12.0);
        definition.node.flags.set_unshaded(true);
        let unshaded = ParticleEmitterUniform::new(&definition, &GlobalTransform::default(), 12.0);
        assert_eq!(shaded.render_flags[0], 0);
        assert_eq!(unshaded.render_flags[0], 1);
        assert_eq!(shaded.clock_tail, unshaded.clock_tail);
    }

    #[test]
    fn analytic_motion_and_model_space_follow_current_transform() {
        let record = ParticleInstance::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            2.0,
            10.0,
            5.0,
            Vec3::ONE,
            false,
        );
        let mut definition = emitter();
        let transform = GlobalTransform::from_translation(Vec3::new(100.0, 200.0, 300.0));
        let uniform = ParticleEmitterUniform::new(&definition, &transform, 12.0);
        assert_eq!(record.center(&uniform), Vec3::new(9.0, 12.0, 11.0));
        definition.node.flags.set_model_space(true);
        let uniform = ParticleEmitterUniform::new(&definition, &transform, 12.0);
        assert_eq!(record.center(&uniform), Vec3::new(109.0, 212.0, 311.0));
    }

    #[test]
    fn split_clock_preserves_short_ages_after_long_runtime() {
        let birth = 1_000_000_000.0;
        let record = ParticleInstance::new(Vec3::ZERO, Vec3::X, 0.0, birth, 1.0, Vec3::ONE, false);
        let uniform =
            ParticleEmitterUniform::new(&emitter(), &GlobalTransform::default(), birth + 0.125);
        assert_eq!(record.center(&uniform), Vec3::new(0.125, 0.0, 0.0));
    }

    #[test]
    fn sorting_uses_analytic_camera_depth_for_each_view() {
        let records = vec![
            ParticleInstance::new(
                Vec3::new(100.0, 0.0, -1.0),
                Vec3::ZERO,
                0.0,
                0.0,
                5.0,
                Vec3::ONE,
                false,
            ),
            ParticleInstance::new(
                Vec3::ZERO,
                Vec3::new(0.0, 0.0, -2.0),
                0.0,
                0.0,
                5.0,
                Vec3::ONE,
                false,
            ),
        ];
        let uniform = ParticleEmitterUniform::new(&emitter(), &GlobalTransform::default(), 1.0);
        let data = instances(records, uniform);
        assert_eq!(sorted_indices(&data, &Affine3A::IDENTITY), vec![1, 0]);
        let reverse = Affine3A::from_quat(Quat::from_rotation_y(PI));
        assert_eq!(sorted_indices(&data, &reverse), vec![0, 1]);
    }

    #[test]
    fn particle_shader_validates_and_matches_host_buffer_layouts() {
        let shader = include_str!("../shaders/wc3_particle.wgsl");
        // Bevy supplies these imports. Stubs let Naga validate the complete
        // particle shader without starting a renderer or requiring a GPU.
        let body = &shader[shader.find("@group(3)").unwrap()..];
        let imports = r#"
struct View { world_from_view: mat4x4<f32>, clip_from_view: mat4x4<f32> };
@group(0) @binding(0) var<uniform> view: View;
fn position_world_to_clip(p: vec3<f32>) -> vec4<f32> { return vec4(p, 1.0); }
struct Material { base_color: vec4<f32>, reflectance: vec3<f32>, perceptual_roughness: f32 };
struct Lighting {
    material: Material, frag_coord: vec4<f32>, world_position: vec4<f32>,
    world_normal: vec3<f32>, N: vec3<f32>, V: vec3<f32>, is_orthographic: bool,
};
fn pbr_input_new() -> Lighting { var result: Lighting; return result; }
fn calculate_view(p: vec4<f32>, orthographic: bool) -> vec3<f32> { return vec3(0.0, 0.0, 1.0); }
fn apply_pbr_lighting(p: Lighting) -> vec4<f32> { return p.material.base_color; }
"#;
        for alpha_key in [false, true] {
            let mut enabled = true;
            let mut source = imports.to_owned();
            for line in body.lines() {
                match line.trim() {
                    "#ifdef ALPHA_KEY" => enabled = alpha_key,
                    "#endif" => enabled = true,
                    _ if enabled => {
                        source.push_str(line);
                        source.push('\n');
                    }
                    _ => {}
                }
            }
            let module = parse_str(&source)
                .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
            Validator::new(ValidationFlags::all(), Capabilities::all())
                .validate(&module)
                .unwrap();
            for (name, size) in [
                ("Particle", size_of::<ParticleInstance>()),
                ("Emitter", size_of::<ParticleEmitterUniform>()),
            ] {
                let ty = module
                    .types
                    .iter()
                    .find(|(_, ty)| ty.name.as_deref() == Some(name))
                    .unwrap()
                    .1;
                let TypeInner::Struct { span, .. } = ty.inner else {
                    panic!("expected struct")
                };
                assert_eq!(span as usize, size);
            }
        }
    }
}
