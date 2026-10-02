//! GPU allocation and upload mechanics shared by quads and ribbon passes.
use bevy::prelude::Component;
use bevy::render::render_resource::{
    BindGroup, BindGroupEntries, BindGroupLayout, Buffer, BufferDescriptor, BufferInitDescriptor,
    BufferUsages,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::view::RetainedViewEntity;
use bytemuck::{bytes_of, cast_slice, Pod};
use std::collections::HashMap;
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
