//! GPU buffers, binding layouts, and draw plumbing shared by quads and ribbons.
use bevy::core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d};
use bevy::mesh::VertexBufferLayout;
use bevy::prelude::*;
use bevy::render::render_phase::{DrawFunctionId, PhaseItemExtraIndex};
use bevy::render::render_resource::{
    binding_types::{sampler, storage_buffer_read_only_sized, texture_2d, uniform_buffer_sized},
    BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntries, Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages,
    CachedRenderPipelineId, SamplerBindingType, ShaderStages, TextureSampleType, VertexAttribute,
    VertexFormat, VertexStepMode,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::sync_world::MainEntity;
use bevy::render::texture::GpuImage;
use bevy::render::view::{NoIndirectDrawing, RetainedViewEntity};
use bytemuck::{bytes_of, cast_slice, Pod};
use std::collections::HashMap;
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

#[derive(Component)]
pub(crate) struct EffectBuffer<I: Send + Sync + 'static> {
    pub(crate) records: ResidentRecordBuffer,
    pub(crate) live_indices: Arc<Vec<I>>,
    uniform: Buffer,
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
        let PassResources {
            device,
            queue,
            layout,
            image,
        } = resources;
        let uniform = if let Some(previous) = previous {
            queue.write_buffer(&previous.uniform, 0, bytes_of(uniform_data));
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
        let texture = device.create_bind_group(
            "wc3 effect data",
            layout,
            &BindGroupEntries::sequential((
                &image.texture_view,
                &image.sampler,
                uniform.as_entire_binding(),
                records.buffer.as_entire_binding(),
            )),
        );
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
            order,
            texture,
            sorted,
        }
    }
}

/// Effect draw commands need direct mesh bindings from Bevy's GPU preprocessing.
pub(crate) fn configure_effect_views(
    mut commands: Commands,
    cameras: Query<Entity, (With<Camera3d>, Without<NoIndirectDrawing>)>,
) {
    for camera in &cameras {
        commands.entity(camera).insert(NoIndirectDrawing);
    }
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
