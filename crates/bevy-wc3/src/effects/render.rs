//! GPU buffers, binding layouts, and draw plumbing shared by quads and ribbons.
use bevy::core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d};
use bevy::ecs::system::{
    lifetimeless::{Read, SRes},
    SystemParamItem,
};
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
use std::collections::HashMap;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::sync::Arc;

use crate::effects::records::{upload_records, RecordRing, RingCursor};

#[derive(Clone)]
pub(crate) struct ResidentRecordBuffer {
    pub(crate) buffer: Buffer,
    capacity: usize,
    cursor: RingCursor,
}
impl ResidentRecordBuffer {
    /// Upload exactly sampled live records without allocating per-frame meshes.
    /// Immutable birth transports continue to use their write cursors instead.
    pub(crate) fn prepare_slice<R: Pod>(
        values: &[R],
        previous: Option<&Self>,
        device: &RenderDevice,
        queue: &RenderQueue,
        changed: bool,
    ) -> Self {
        if let Some(previous) = previous.filter(|p| p.capacity >= values.len()) {
            if changed && !values.is_empty() {
                queue.write_buffer(&previous.buffer, 0, cast_slice(values));
            }
            return previous.clone();
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
            cursor: RingCursor::default(),
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
                    Some(previous.cursor),
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
            cursor: records.cursor(),
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
    pub(crate) live_indices: Arc<Vec<I>>,
    uniform: Buffer,
    uniform_bytes: Arc<[u8]>,
    binding: EffectBinding,
    pub(crate) order: DrawOrderBuffer,
    pub(crate) texture: BindGroup,
    pub(crate) sorted: HashMap<RetainedViewEntity, DrawOrderBuffer>,
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
        previous: Option<&Self>,
        resources: PassResources<'_>,
        sorted_orders: impl IntoIterator<Item = (RetainedViewEntity, Vec<I>)>,
    ) -> Self {
        Self::prepare_layers(
            records,
            indices,
            uniform_data,
            previous,
            resources,
            sorted_orders,
            None,
        )
    }

    pub(crate) fn prepare_layers<U: Pod>(
        records: ResidentRecordBuffer,
        indices: &Arc<Vec<I>>,
        uniform_data: &U,
        previous: Option<&Self>,
        resources: PassResources<'_>,
        sorted_orders: impl IntoIterator<Item = (RetainedViewEntity, Vec<I>)>,
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
            .filter(|previous| previous.uniform_bytes.as_ref() == bytes)
            .map_or_else(
                || Arc::from(bytes),
                |previous| previous.uniform_bytes.clone(),
            );
        let uniform = if let Some(previous) = previous {
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
            previous.map(|p| &p.order),
            device,
            queue,
            previous.is_none_or(|p| !Arc::ptr_eq(&p.live_indices, indices)),
        );
        let binding = EffectBinding {
            layout: layout.id(),
            image: (image.texture_view.id(), image.sampler.id()),
            extra_images: extra_images
                .map(|images| images.map(|image| (image.texture_view.id(), image.sampler.id()))),
            uniform: uniform.id(),
            records: records.buffer.id(),
        };
        let texture = if let Some(previous) = previous.filter(|p| p.binding == binding) {
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
        let sorted = sorted_orders
            .into_iter()
            .map(|(view, indices)| {
                let buffer = DrawOrderBuffer::prepare(
                    &indices,
                    previous.and_then(|p| p.sorted.get(&view)),
                    device,
                    queue,
                    true,
                );
                (view, buffer)
            })
            .collect();
        Self {
            records,
            live_indices: indices.clone(),
            uniform,
            uniform_bytes,
            binding,
            order,
            texture,
            sorted,
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
    type ViewQuery = Read<ExtractedView>;
    type ItemQuery = Read<EffectBuffer<I>>;
    fn render<'w>(
        item: &P,
        view: &'w ExtractedView,
        buffer: Option<&'w EffectBuffer<I>>,
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
