//! GPU ribbon quads built from pairs of immutable world-space cross-sections.
use crate::effects::blend::EffectBlend;
use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::math::{Affine3A, Vec3A};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    MeshPipeline, MeshPipelineKey, MeshPipelineSystems, RenderMeshInstances, SetMeshViewBindGroup,
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
use std::collections::HashMap;

use super::records::{
    segment_center, RibbonInstances, RibbonSection, RibbonSegment, RibbonUniform,
};
use crate::effects::render::{
    effect_texture_layout, effect_vertex_layout, effect_visible, transparent_effect_draw,
    DrawEffectInstances, EffectBuffer, PassResources, ResidentRecordBuffer,
};

pub(crate) struct RibbonRenderPlugin;
impl Plugin for RibbonRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<RibbonInstances>::default());
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
    let texture_layout =
        effect_texture_layout::<RibbonUniform, RibbonSection>("wc3 ribbon texture layout");
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
        descriptor
            .vertex
            .buffers
            .push(effect_vertex_layout::<RibbonSegment>(
                VertexFormat::Uint32x4,
            ));
        // The shader reads emitter-owned transforms and birth records, so it
        // needs only Bevy's view groups, not a preprocessed mesh bind group.
        descriptor.layout.truncate(2);
        descriptor.layout.push(self.texture_layout.clone());
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = self.shader.clone();
            if let Some(Some(target)) = fragment.targets.first_mut() {
                let (src, dst) = match key.filter {
                    3 => (BlendFactor::One, BlendFactor::One),
                    4 => (BlendFactor::SrcAlpha, BlendFactor::One),
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
fn filter_key(filter: EffectBlend) -> u8 {
    match filter {
        EffectBlend::Opaque => 0,
        EffectBlend::Cutout => 1,
        EffectBlend::Alpha => 2,
        EffectBlend::Additive => 3,
        EffectBlend::AdditiveAlpha => 4,
        EffectBlend::Modulate => 5,
        EffectBlend::Modulate2x => 6,
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
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    view_keys: Res<ViewKeyCache>,
) {
    let draw = draws.read().id::<DrawRibbons>();
    let mut ordered: Vec<_> = ribbons.iter().collect();
    ordered.sort_by_key(|(_, _, data)| (data.emitter.to_bits(), data.layer_index));
    for (view, visible) in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
            continue;
        };
        let Some(&view_key) = view_keys.get(&view.retained_view_entity) else {
            continue;
        };
        for &(entity, main_entity, data) in &ordered {
            if data.live_indices.is_empty() || !effect_visible(visible, *main_entity) {
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
    query: Query<(Entity, &MainEntity, &RibbonInstances, Option<&RibbonBuffer>)>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<RibbonPipeline>,
    images: Res<RenderAssets<GpuImage>>,
    fallback: Res<FallbackImage>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    mut scratch: Local<Vec<RibbonSection>>,
) {
    // All material layers of an emitter share the same resident section buffer.
    // Uniforms, texture bindings, and draw ordering remain specific to each pass.
    let mut shared_sections: HashMap<Entity, ResidentRecordBuffer> = HashMap::new();
    for (entity, main_entity, data, previous) in &query {
        if data.live_indices.is_empty()
            || !views
                .iter()
                .any(|(_, visible)| effect_visible(visible, *main_entity))
        {
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
            .filter(|(_, visible)| {
                (data.sort_far || data.sort_near) && effect_visible(visible, *main_entity)
            })
            .map(|(view, _)| {
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
    DrawEffectInstances<RibbonBuffer>,
);
#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
