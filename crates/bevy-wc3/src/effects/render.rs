//! GPU buffers, binding layouts, and draw plumbing shared by quads and ribbons.
use crate::effects::age_sprites_gpu::immutable::BirthSnapshot;
use crate::effects::uploads::{EffectUploadStats, EffectUploadTraffic};
#[path = "view_orders.rs"]
mod view_orders;
use bevy::core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d};
use bevy::ecs::system::{
    lifetimeless::{Read, SRes},
    SystemParamItem,
};
use bevy::math::Affine3A;
use bevy::mesh::VertexBufferLayout;
use bevy::pbr::MeshPipelineKey;
#[path = "geometry.rs"]
mod geometry;
use bevy::prelude::*;
use bevy::render::mesh::{allocator::MeshAllocator, RenderMesh, RenderMeshBufferInfo};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{DrawFunctionId, PhaseItemExtraIndex};
use bevy::render::render_phase::{
    PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass,
};
use bevy::render::render_resource::{
    binding_types::{sampler, storage_buffer_read_only_sized, texture_2d, uniform_buffer_sized},
    BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntries, BindGroupLayoutId, Buffer, BufferDescriptor, BufferId,
    BufferInitDescriptor, BufferUsages, CachedRenderPipelineId, SamplerBindingType, SamplerId,
    ShaderStages, TextureSampleType, TextureViewId, VertexAttribute, VertexFormat, VertexStepMode,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::sync_world::MainEntity;
use bevy::render::texture::GpuImage;
use bevy::render::view::ExtractedView;
use bevy::render::view::{RenderVisibleEntities, RetainedViewEntity};
use bytemuck::{bytes_of, cast_slice, Pod};
pub(crate) use geometry::{
    effect_capsule, effect_quad, EffectGeometry, EffectGeometryPlugin, ExtractedEffectGeometry,
};
use std::any::Any;
use std::marker::PhantomData;
use std::mem::{size_of_val, take};
use std::num::NonZeroU64;
use std::ops::Range;
use std::sync::Arc;
pub(crate) use view_orders::ViewOrders;
pub(crate) use view_orders::{SortPolicy, SortScratch};

use crate::effects::records::{upload_records, RecordRing, RingCursor};

#[derive(Clone)]
pub(crate) struct ResidentRecordBuffer {
    pub(crate) buffer: Buffer,
    capacity_bytes: usize,
    source: RecordSource,
}
#[derive(Clone)]
enum RecordSource {
    Ring {
        cursor: RingCursor,
        lineage: Arc<()>,
    },
    Sampled(Arc<dyn Any + Send + Sync>),
    Births(Arc<BirthSnapshot>),
}
impl RecordSource {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Ring {
                    cursor: a,
                    lineage: x,
                },
                Self::Ring {
                    cursor: b,
                    lineage: y,
                },
            ) => a == b && Arc::ptr_eq(x, y),
            (Self::Sampled(a), Self::Sampled(b)) => Arc::ptr_eq(a, b),
            (Self::Births(a), Self::Births(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
/// Visit word ranges, combining adjacent differing fixed-stride birth slots.
/// No history is required: compare with the exact resident snapshot or initialize
/// the full block when lineage, semantic stride or capacity made it unavailable.
fn changed_birth_ranges(
    current: &[u32],
    previous: Option<&[u32]>,
    stride: usize,
    mut upload: impl FnMut(Range<usize>),
) {
    if current.is_empty() {
        return;
    }
    assert!(stride > 0 && current.len().is_multiple_of(stride));
    let Some(previous) = previous.filter(|old| old.len() == current.len()) else {
        upload(0..current.len());
        return;
    };
    let mut start = None;
    for (index, (new, old)) in current
        .chunks_exact(stride)
        .zip(previous.chunks_exact(stride))
        .enumerate()
    {
        let offset = index * stride;
        if new != old {
            start.get_or_insert(offset);
        } else if let Some(start) = start.take() {
            upload(start..offset);
        }
    }
    if let Some(start) = start {
        upload(start..current.len());
    }
}

impl ResidentRecordBuffer {
    /// Dirty block uploads for arbitrary stable birth slots, not FIFO retirement.
    pub(crate) fn prepare_births(
        values: &Arc<BirthSnapshot>,
        previous: Option<&Self>,
        device: &RenderDevice,
        queue: &RenderQueue,
        uploads: &mut EffectUploadTraffic,
    ) -> Self {
        let words = values.word_count();
        let reusable = previous.filter(|p| p.capacity_bytes >= words * 4);
        let (buffer, capacity_bytes) = reusable.map_or_else(
            || {
                let capacity = words.max(1).next_power_of_two();
                (
                    device.create_buffer(&BufferDescriptor {
                        label: Some("wc3 immutable appearance births"),
                        size: (capacity * 4) as u64,
                        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                    capacity * 4,
                )
            },
            |p| (p.buffer.clone(), p.capacity_bytes),
        );
        let old = reusable.and_then(|p| match &p.source {
            RecordSource::Births(old)
                if Arc::ptr_eq(&old.lineage, &values.lineage)
                    && old.words_per_slot == values.words_per_slot
                    && old.capacity == values.capacity =>
            {
                Some(old)
            }
            _ => None,
        });
        if !old.is_some_and(|old| Arc::ptr_eq(old, values)) {
            for (index, block) in values.blocks.iter().enumerate() {
                let previous_block = old.and_then(|old| old.blocks.get(index));
                if previous_block.is_some_and(|old| Arc::ptr_eq(old, block)) {
                    continue;
                }
                // The resident snapshot is the actual last uploaded contents,
                // not merely the previous simulation/extraction observation.
                changed_birth_ranges(
                    block,
                    previous_block.map(|old| old.as_slice()),
                    values.words_per_slot,
                    |range| {
                        let offset = values.block_word_offset(index) + range.start;
                        let words = &block[range];
                        queue.write_buffer(
                            &buffer,
                            (offset * size_of::<u32>()) as u64,
                            cast_slice(words),
                        );
                        uploads.queued(size_of_val(words));
                    },
                );
            }
        }
        Self {
            buffer,
            capacity_bytes,
            source: RecordSource::Births(values.clone()),
        }
    }
    /// Upload exactly sampled live records without allocating per-frame meshes.
    /// Immutable birth transports continue to use their write cursors instead.
    pub(crate) fn prepare_slice<R: Pod + Send + Sync>(
        values: &Arc<Vec<R>>,
        previous: Option<&Self>,
        device: &RenderDevice,
        queue: &RenderQueue,
        uploads: &mut EffectUploadTraffic,
    ) -> Self {
        let source = RecordSource::Sampled(values.clone());
        if let Some(previous) =
            previous.filter(|p| p.capacity_bytes >= values.len() * size_of::<R>())
        {
            if !previous.source.same(&source) && !values.is_empty() {
                queue.write_buffer(&previous.buffer, 0, cast_slice(values));
                uploads.queued(values.len() * size_of::<R>());
            }
            return Self {
                buffer: previous.buffer.clone(),
                capacity_bytes: previous.capacity_bytes,
                source,
            };
        }
        let capacity = values.len().max(1).next_power_of_two();
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("wc3 sampled effect records"),
            size: (capacity * size_of::<R>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if !values.is_empty() {
            queue.write_buffer(&buffer, 0, cast_slice(values));
            uploads.queued(values.len() * size_of::<R>());
        }
        Self {
            buffer,
            capacity_bytes: capacity * size_of::<R>(),
            source,
        }
    }

    pub(crate) fn prepare<R: Pod>(
        records: &RecordRing<R>,
        previous: Option<&Self>,
        device: &RenderDevice,
        queue: &RenderQueue,
        scratch: &mut Vec<R>,
        uploads: &mut EffectUploadTraffic,
    ) -> Self {
        let (buffer, capacity_bytes, cursor) = if let Some(previous) =
            previous.filter(|p| p.capacity_bytes >= records.capacity() * size_of::<R>())
        {
            (
                previous.buffer.clone(),
                previous.capacity_bytes,
                match &previous.source {
                    RecordSource::Ring { cursor, lineage }
                        if Arc::ptr_eq(lineage, records.lineage()) =>
                    {
                        Some(*cursor)
                    }
                    _ => None,
                },
            )
        } else {
            let capacity = records.capacity();
            let buffer = device.create_buffer(&BufferDescriptor {
                label: Some("wc3 immutable effect records"),
                size: (capacity * size_of::<R>()) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            (buffer, capacity * size_of::<R>(), None)
        };
        upload_records(records, cursor, &buffer, queue, scratch, uploads);
        Self {
            buffer,
            capacity_bytes,
            source: RecordSource::Ring {
                cursor: records.cursor(),
                lineage: records.lineage().clone(),
            },
        }
    }
}

pub(crate) struct DrawOrderBuffer {
    pub(crate) buffer: Buffer,
    capacity: usize,
}
impl DrawOrderBuffer {
    fn prepare<I: Pod>(
        indices: &[I],
        previous: Option<&Self>,
        device: &RenderDevice,
        queue: &RenderQueue,
        changed: bool,
        uploads: &mut EffectUploadTraffic,
    ) -> Self {
        if let Some(previous) = previous.filter(|p| p.capacity >= indices.len()) {
            if changed {
                queue.write_buffer(&previous.buffer, 0, cast_slice(indices));
                uploads.queued(size_of_val(indices));
            }
            return Self {
                buffer: previous.buffer.clone(),
                capacity: previous.capacity,
            };
        }
        let capacity = indices.len().next_power_of_two();
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("wc3 effect draw order"),
            size: (capacity * size_of::<I>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buffer, 0, cast_slice(indices));
        uploads.queued(size_of_val(indices));
        Self { buffer, capacity }
    }
}

/// The resources referenced by a binding, independently of their current contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EffectBinding {
    layout: BindGroupLayoutId,
    image: (TextureViewId, SamplerId),
    extra_images: Option<[(TextureViewId, SamplerId); 2]>,
    uniform: BufferId,
    records: BufferId,
}

#[derive(Component)]
pub(crate) struct EffectBuffer<I: Send + Sync + 'static> {
    pub(crate) records: ResidentRecordBuffer,
    draw: DrawState<I>,
    uniform: Buffer,
    uniform_bytes: Arc<[u8]>,
    center: CenterDependency,
    binding: EffectBinding,
    pub(crate) order: DrawOrderBuffer,
    pub(crate) texture: BindGroup,
    pub(crate) sorted: ViewOrders<RetainedViewEntity, DrawOrderBuffer>,
}

/// The exact inputs used by an effect family's current CPU sorting center.
/// Other families retain their conservative full-uniform snapshot dependency.
#[derive(Clone)]
pub(crate) enum CenterDependency {
    UniformSnapshot(Arc<[u8]>),
    SampledCenters(Arc<Vec<Vec3>>),
    AnalyticParticle {
        clock: Option<[u32; 2]>,
        frame: Option<[[u32; 4]; 4]>,
    },
}
impl CenterDependency {
    pub(crate) fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UniformSnapshot(a), Self::UniformSnapshot(b)) => Arc::ptr_eq(a, b),
            (Self::SampledCenters(a), Self::SampledCenters(b)) => Arc::ptr_eq(a, b),
            (
                Self::AnalyticParticle {
                    clock: a,
                    frame: af,
                },
                Self::AnalyticParticle {
                    clock: b,
                    frame: bf,
                },
            ) => a == b && af == bf,
            _ => false,
        }
    }
}

struct OrderRevision<'a, I> {
    records: &'a RecordSource,
    indices: &'a Arc<Vec<I>>,
    center: &'a CenterDependency,
}
impl<I> OrderRevision<'_, I> {
    fn same(&self, other: &Self) -> bool {
        self.records.same(other.records)
            && Arc::ptr_eq(self.indices, other.indices)
            && self.center.same(other.center)
    }
}

/// Residency and source identity survive frames which have no active draws.
struct DrawState<I> {
    source: Arc<Vec<I>>,
    instances: u32,
}
impl<I> DrawState<I> {
    fn new(source: &Arc<Vec<I>>) -> Self {
        Self {
            source: source.clone(),
            instances: source.len() as u32,
        }
    }
    fn suspend(&mut self) {
        self.instances = 0;
    }
}

pub(crate) struct PassResources<'a> {
    pub(crate) device: &'a RenderDevice,
    pub(crate) queue: &'a RenderQueue,
    pub(crate) layout: &'a BindGroupLayout,
    pub(crate) image: &'a GpuImage,
    pub(crate) uploads: &'a mut EffectUploadStats,
}
impl<I: Pod + Send + Sync> EffectBuffer<I> {
    pub(crate) fn prepare<U: Pod>(
        records: ResidentRecordBuffer,
        indices: &Arc<Vec<I>>,
        uniform_data: &U,
        previous: Option<&mut Self>,
        resources: PassResources<'_>,
    ) -> Self {
        Self::prepare_layers(records, indices, uniform_data, previous, resources, None)
    }

    pub(crate) fn prepare_layers<U: Pod>(
        records: ResidentRecordBuffer,
        indices: &Arc<Vec<I>>,
        uniform_data: &U,
        previous: Option<&mut Self>,
        resources: PassResources<'_>,
        extra_images: Option<[&GpuImage; 2]>,
    ) -> Self {
        Self::prepare_center_layers(
            records,
            indices,
            uniform_data,
            previous,
            resources,
            extra_images,
            None,
        )
    }

    pub(crate) fn prepare_center<U: Pod>(
        records: ResidentRecordBuffer,
        indices: &Arc<Vec<I>>,
        uniform_data: &U,
        previous: Option<&mut Self>,
        resources: PassResources<'_>,
        center: CenterDependency,
    ) -> Self {
        Self::prepare_center_layers(
            records,
            indices,
            uniform_data,
            previous,
            resources,
            None,
            Some(center),
        )
    }

    pub(crate) fn prepare_center_layers<U: Pod>(
        records: ResidentRecordBuffer,
        indices: &Arc<Vec<I>>,
        uniform_data: &U,
        previous: Option<&mut Self>,
        resources: PassResources<'_>,
        extra_images: Option<[&GpuImage; 2]>,
        center: Option<CenterDependency>,
    ) -> Self {
        Self::prepare_center_layers_bytes(
            records,
            indices,
            bytes_of(uniform_data),
            previous,
            resources,
            extra_images,
            center,
        )
    }

    pub(crate) fn prepare_center_layers_bytes(
        records: ResidentRecordBuffer,
        indices: &Arc<Vec<I>>,
        bytes: &[u8],
        previous: Option<&mut Self>,
        resources: PassResources<'_>,
        extra_images: Option<[&GpuImage; 2]>,
        center: Option<CenterDependency>,
    ) -> Self {
        let PassResources {
            device,
            queue,
            layout,
            image,
            uploads,
        } = resources;
        let uniform_bytes = previous
            .as_deref()
            .filter(|previous| previous.uniform_bytes.as_ref() == bytes)
            .map_or_else(
                || Arc::from(bytes),
                |previous| previous.uniform_bytes.clone(),
            );
        let uniform = if let Some(previous) = previous
            .as_deref()
            .filter(|previous| previous.uniform_bytes.len() == bytes.len())
        {
            if !Arc::ptr_eq(&uniform_bytes, &previous.uniform_bytes) {
                queue.write_buffer(&previous.uniform, 0, bytes);
                uploads.uniforms.queued(bytes.len());
            }
            previous.uniform.clone()
        } else {
            uploads.uniforms.initialized(bytes.len());
            device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("wc3 effect pass parameters"),
                contents: bytes,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };
        let order = DrawOrderBuffer::prepare(
            indices,
            previous.as_deref().map(|p| &p.order),
            device,
            queue,
            previous
                .as_deref()
                .is_none_or(|p| !Arc::ptr_eq(&p.draw.source, indices)),
            &mut uploads.natural_indices,
        );
        let binding = EffectBinding {
            layout: layout.id(),
            image: (image.texture_view.id(), image.sampler.id()),
            extra_images: extra_images
                .map(|images| images.map(|image| (image.texture_view.id(), image.sampler.id()))),
            uniform: uniform.id(),
            records: records.buffer.id(),
        };
        let texture = if let Some(previous) = previous.as_deref().filter(|p| p.binding == binding) {
            previous.texture.clone()
        } else if let Some([secondary, tertiary]) = extra_images {
            device.create_bind_group(
                "wc3 layered effect data",
                layout,
                &BindGroupEntries::sequential((
                    &image.texture_view,
                    &image.sampler,
                    uniform.as_entire_binding(),
                    records.buffer.as_entire_binding(),
                    &secondary.texture_view,
                    &secondary.sampler,
                    &tertiary.texture_view,
                    &tertiary.sampler,
                )),
            )
        } else {
            device.create_bind_group(
                "wc3 effect data",
                layout,
                &BindGroupEntries::sequential((
                    &image.texture_view,
                    &image.sampler,
                    uniform.as_entire_binding(),
                    records.buffer.as_entire_binding(),
                )),
            )
        };
        let center =
            center.unwrap_or_else(|| CenterDependency::UniformSnapshot(uniform_bytes.clone()));
        let revision = OrderRevision {
            records: &records.source,
            indices,
            center: &center,
        };
        let changed = previous.as_deref().is_none_or(|p| {
            !revision.same(&OrderRevision {
                records: &p.records.source,
                indices: &p.draw.source,
                center: &p.center,
            })
        });
        let mut sorted = previous.map(|p| take(&mut p.sorted)).unwrap_or_default();
        if changed {
            sorted.invalidate();
        }
        Self {
            records,
            draw: DrawState::new(indices),
            uniform,
            uniform_bytes,
            center,
            binding,
            order,
            texture,
            sorted,
        }
    }
    pub(crate) fn suspend_draws(&mut self) {
        self.draw.suspend();
    }
    pub(crate) fn retain_order_views(&mut self, live: impl FnMut(RetainedViewEntity) -> bool) {
        self.sorted.retain(live);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_orders(
        &mut self,
        views: impl Iterator<Item = (RetainedViewEntity, Affine3A, bool)>,
        policy: SortPolicy,
        scratch: &mut SortScratch<I>,
        device: &RenderDevice,
        queue: &RenderQueue,
        uploads: &mut EffectUploadTraffic,
        mut sort: impl FnMut(&Affine3A, &mut SortScratch<I>),
    ) {
        self.sorted.set_policy(policy);
        if policy == SortPolicy::None {
            self.sorted.clear();
            return;
        }
        for (view, affine, visible) in views {
            if !visible {
                continue;
            }
            self.sorted.prepare(view, affine, |previous| {
                scratch.clear();
                sort(&affine, scratch);
                DrawOrderBuffer::prepare(&scratch.order, previous, device, queue, true, uploads)
            });
        }
    }
}

/// Reuse native per-view procedural visibility, including render layers.
pub(crate) fn effect_visible(visible: &RenderVisibleEntities, entity: MainEntity) -> bool {
    visible.get::<EffectGeometry>().is_some_and(|meshes| {
        meshes.entities_gpu_culling.contains_key(&entity)
            || meshes
                .entities_cpu_culling
                .binary_search_by_key(&entity, |(_, main)| *main)
                .is_ok()
    })
}

/// Resolve immutable vertex/index geometry without creating a mesh uniform or preprocessing row.
pub(crate) fn effect_mesh<'a>(
    geometry: &ExtractedEffectGeometry,
    meshes: &'a RenderAssets<RenderMesh>,
    view_key: MeshPipelineKey,
) -> Option<(&'a RenderMesh, MeshPipelineKey)> {
    let mesh = meshes.get(geometry.mesh)?;
    let key = view_key
        | MeshPipelineKey::from_primitive_topology_and_strip_index(
            mesh.primitive_topology(),
            mesh.index_format(),
        );
    Some((mesh, key))
}

/// Both effect shaders bind a texture, sampler, uniform, and resident birth records.
pub(crate) fn effect_texture_layout<U: Pod, R: Pod>(
    label: &'static str,
) -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        label,
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer_sized(false, NonZeroU64::new(size_of::<U>() as u64)),
                storage_buffer_read_only_sized(false, NonZeroU64::new(size_of::<R>() as u64)),
            ),
        ),
    )
}

/// Instance attributes select records without changing the shared quad mesh.
pub(crate) fn effect_vertex_layout<I: Pod>(format: VertexFormat) -> VertexBufferLayout {
    VertexBufferLayout {
        array_stride: size_of::<I>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: vec![VertexAttribute {
            format,
            offset: 0,
            shader_location: 3,
        }],
    }
}

/// Requeue transient draws each frame so empty or hidden emitters cannot retain stale buffers.
pub(crate) fn transparent_effect_draw(
    entity: Entity,
    main_entity: MainEntity,
    pipeline: CachedRenderPipelineId,
    draw_function: DrawFunctionId,
    center: Vec3,
    depth_bias: f32,
) -> Transparent3d {
    Transparent3d {
        sorting_info: TransparentSortingInfo3d::Sorted {
            mesh_center: center,
            depth_bias,
        },
        entity: (entity, main_entity),
        pipeline,
        draw_function,
        distance: 0.0,
        batch_range: 0..1,
        extra_index: PhaseItemExtraIndex::None,
        indexed: true,
    }
}

pub(crate) struct DrawEffectInstances<B>(PhantomData<B>);
impl<P: PhaseItem, I: Send + Sync + 'static> RenderCommand<P>
    for DrawEffectInstances<EffectBuffer<I>>
{
    type Param = (SRes<RenderAssets<RenderMesh>>, SRes<MeshAllocator>);
    type ViewQuery = (Read<ExtractedView>, Read<RenderVisibleEntities>);
    type ItemQuery = (Read<EffectBuffer<I>>, Read<ExtractedEffectGeometry>);
    fn render<'w>(
        item: &P,
        (view, visible): (&'w ExtractedView, &'w RenderVisibleEntities),
        item_data: Option<(&'w EffectBuffer<I>, &'w ExtractedEffectGeometry)>,
        (meshes, allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((buffer, geometry)) = item_data.filter(|(buffer, _)| buffer.draw.instances != 0)
        else {
            return RenderCommandResult::Skip;
        };
        if !effect_visible(visible, item.main_entity()) {
            return RenderCommandResult::Skip;
        }
        let allocator = allocator.into_inner();
        let Some(mesh) = meshes.into_inner().get(geometry.mesh) else {
            return RenderCommandResult::Skip;
        };
        let Some(vertex) = allocator.mesh_vertex_slice(&geometry.mesh) else {
            return RenderCommandResult::Skip;
        };
        pass.set_vertex_buffer(0, vertex.buffer.slice(..));
        let records = buffer
            .sorted
            .get(&view.retained_view_entity)
            .unwrap_or(&buffer.order);
        pass.set_vertex_buffer(1, records.buffer.slice(..));
        pass.set_bind_group(2, &buffer.texture, &[]);
        match &mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count,
            } => {
                let Some(index) = allocator.mesh_index_slice(&geometry.mesh) else {
                    return RenderCommandResult::Skip;
                };
                pass.set_index_buffer(index.buffer.slice(..), *index_format);
                pass.draw_indexed(
                    index.range.start..index.range.start + count,
                    vertex.range.start as i32,
                    0..buffer.draw.instances,
                );
            }
            RenderMeshBufferInfo::NonIndexed => pass.draw(vertex.range, 0..buffer.draw.instances),
        }

        RenderCommandResult::Success
    }
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
