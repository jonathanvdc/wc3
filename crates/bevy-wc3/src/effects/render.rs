//! GPU buffers, binding layouts, and draw plumbing shared by quads and ribbons.
#[path = "view_orders.rs"]
mod view_orders;
use bevy::core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d};
use bevy::ecs::system::{
    lifetimeless::{Read, SRes},
    SystemParamItem,
};
use bevy::math::Affine3A;
use bevy::mesh::VertexBufferLayout;
use bevy::pbr::RenderMeshInstances;
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
use std::any::Any;
use std::marker::PhantomData;
use std::mem::take;
use std::num::NonZeroU64;
use std::sync::Arc;
use view_orders::ViewOrders;
pub(crate) use view_orders::{SortPolicy, SortScratch};

use crate::effects::records::{upload_records, RecordRing, RingCursor};

#[derive(Clone)]
pub(crate) struct ResidentRecordBuffer {
    pub(crate) buffer: Buffer,
    capacity: usize,
    source: RecordSource,
}
#[derive(Clone)]
enum RecordSource {
    Ring {
        cursor: RingCursor,
        lineage: Arc<()>,
    },
    Sampled(Arc<dyn Any + Send + Sync>),
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
            _ => false,
        }
    }
}
impl ResidentRecordBuffer {
    /// Upload exactly sampled live records without allocating per-frame meshes.
    /// Immutable birth transports continue to use their write cursors instead.
    pub(crate) fn prepare_slice<R: Pod + Send + Sync>(
        values: &Arc<Vec<R>>,
        previous: Option<&Self>,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Self {
        let source = RecordSource::Sampled(values.clone());
        if let Some(previous) = previous.filter(|p| p.capacity >= values.len()) {
            if !previous.source.same(&source) && !values.is_empty() {
                queue.write_buffer(&previous.buffer, 0, cast_slice(values));
            }
            return Self {
                buffer: previous.buffer.clone(),
                capacity: previous.capacity,
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
        }
        Self {
            buffer,
            capacity,
            source,
        }
    }

    pub(crate) fn prepare<R: Pod>(
        records: &RecordRing<R>,
        previous: Option<&Self>,
        device: &RenderDevice,
        queue: &RenderQueue,
        scratch: &mut Vec<R>,
    ) -> Self {
        let (buffer, capacity, cursor) =
            if let Some(previous) = previous.filter(|p| p.capacity >= records.capacity()) {
                (
                    previous.buffer.clone(),
                    previous.capacity,
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
                (buffer, capacity, None)
            };
        upload_records(records, cursor, &buffer, queue, scratch);
        Self {
            buffer,
            capacity,
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
    ) -> Self {
        if let Some(previous) = previous.filter(|p| p.capacity >= indices.len()) {
            if changed {
                queue.write_buffer(&previous.buffer, 0, cast_slice(indices));
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
    binding: EffectBinding,
    pub(crate) order: DrawOrderBuffer,
    pub(crate) texture: BindGroup,
    pub(crate) sorted: ViewOrders<RetainedViewEntity, DrawOrderBuffer>,
}

struct OrderRevision<'a, I> {
    records: &'a RecordSource,
    indices: &'a Arc<Vec<I>>,
    uniform: &'a Arc<[u8]>,
}
impl<I> OrderRevision<'_, I> {
    fn same(&self, other: &Self) -> bool {
        self.records.same(other.records)
            && Arc::ptr_eq(self.indices, other.indices)
            && Arc::ptr_eq(self.uniform, other.uniform)
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
        mut previous: Option<&mut Self>,
        resources: PassResources<'_>,
        extra_images: Option<[&GpuImage; 2]>,
    ) -> Self {
        let PassResources {
            device,
            queue,
            layout,
            image,
        } = resources;
        let bytes = bytes_of(uniform_data);
        let uniform_bytes = previous
            .as_deref()
            .filter(|previous| previous.uniform_bytes.as_ref() == bytes)
            .map_or_else(
                || Arc::from(bytes),
                |previous| previous.uniform_bytes.clone(),
            );
        let uniform = if let Some(previous) = previous.as_deref() {
            if !Arc::ptr_eq(&uniform_bytes, &previous.uniform_bytes) {
                queue.write_buffer(&previous.uniform, 0, bytes);
            }
            previous.uniform.clone()
        } else {
            device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("wc3 effect pass parameters"),
                contents: bytes_of(uniform_data),
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
        let revision = OrderRevision {
            records: &records.source,
            indices,
            uniform: &uniform_bytes,
        };
        let changed = previous.as_deref().is_none_or(|p| {
            !revision.same(&OrderRevision {
                records: &p.records.source,
                indices: &p.draw.source,
                uniform: &p.uniform_bytes,
            })
        });
        let mut sorted = previous
            .as_deref_mut()
            .map(|p| take(&mut p.sorted))
            .unwrap_or_default();
        if changed {
            sorted.invalidate();
        }
        Self {
            records,
            draw: DrawState::new(indices),
            uniform,
            uniform_bytes,
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

    pub(crate) fn prepare_orders(
        &mut self,
        views: impl Iterator<Item = (RetainedViewEntity, Affine3A, bool)>,
        policy: SortPolicy,
        scratch: &mut SortScratch<I>,
        device: &RenderDevice,
        queue: &RenderQueue,
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
                DrawOrderBuffer::prepare(&scratch.order, previous, device, queue, true)
            });
        }
    }
}

/// Reuse Bevy's per-view mesh selection, including render layers and visibility.
pub(crate) fn effect_visible(visible: &RenderVisibleEntities, entity: MainEntity) -> bool {
    visible.get::<Mesh3d>().is_some_and(|meshes| {
        meshes.entities_gpu_culling.contains_key(&entity)
            || meshes
                .entities_cpu_culling
                .binary_search_by_key(&entity, |(_, main)| *main)
                .is_ok()
    })
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
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = (Read<ExtractedView>, Read<RenderVisibleEntities>);
    type ItemQuery = Read<EffectBuffer<I>>;
    fn render<'w>(
        item: &P,
        (view, visible): (&'w ExtractedView, &'w RenderVisibleEntities),
        buffer: Option<&'w EffectBuffer<I>>,
        (meshes, instances, allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(buffer) = buffer.filter(|buffer| buffer.draw.instances != 0) else {
            return RenderCommandResult::Skip;
        };
        if !effect_visible(visible, item.main_entity()) {
            return RenderCommandResult::Skip;
        }
        let allocator = allocator.into_inner();
        let Some(mesh_instance) = instances.render_mesh_queue_data(item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id()) else {
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
        pass.set_bind_group(2, &buffer.texture, &[]);
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
