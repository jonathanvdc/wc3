//! Conditional per-view refraction using the ordinary particle instance transport.
use bevy::camera::{MainPassResolutionOverride, Viewport};
use bevy::core_pipeline::{
    core_3d::{main_transparent_pass_3d, prepare_core_3d_depth_textures, Transparent3d},
    Core3d, Core3dSystems, FullscreenShader,
};
use bevy::ecs::system::{lifetimeless::Read, SystemParamItem};
use bevy::prelude::*;
use bevy::render::view::RetainedViewEntity;
use bevy::render::{
    camera::ExtractedCamera,
    extract_component::{ExtractComponent, ExtractComponentPlugin},
    render_phase::{
        PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass, ViewSortedRenderPhases,
    },
    render_resource::{
        binding_types::{sampler, texture_2d, uniform_buffer},
        *,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
    texture::{CachedTexture, TextureCache},
    view::{ExtractedView, ViewDepthTexture, ViewTarget},
    Render, RenderApp, RenderStartup, RenderSystems,
};
use std::collections::HashSet;
use std::sync::Mutex;

/// Camera controls for authored particle refraction. Distances are scene units.
/// The default retains the native numerical distance in an unconverted scene.
#[derive(Component, Clone, Copy, Debug, ExtractComponent)]
pub struct ParticleRefraction {
    /// Nonnegative displacement multiplier; zero disables target allocation and composition.
    pub strength: f32,
    /// Positive view-space fade distance in scene units, independent of model scale.
    pub fade_distance: f32,
}
impl Default for ParticleRefraction {
    fn default() -> Self {
        Self {
            strength: 1.,
            fade_distance: 100.,
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct RefractionPhases(pub ViewSortedRenderPhases<Transparent3d>);

pub(super) struct RefractionPlugin;
impl Plugin for RefractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<ParticleRefraction>::default());
        let Some(render) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render
            .init_resource::<RefractionPhases>()
            .init_resource::<SpecializedRenderPipelines<RefractionPipeline>>()
            .add_systems(RenderStartup, initialize)
            .add_systems(
                Render,
                (
                    sort_phases.in_set(RenderSystems::PhaseSort),
                    prepare_views
                        .after(prepare_core_3d_depth_textures)
                        .in_set(RenderSystems::PrepareResources),
                ),
            )
            .add_systems(
                Core3d,
                refract
                    .after(main_transparent_pass_3d)
                    .in_set(Core3dSystems::MainPass),
            );
    }
}

#[derive(Clone, Copy, PartialEq, ShaderType)]
struct RefractionView {
    projection_scale: Vec2,
    strength: f32,
    fade_distance_inverse: f32,
}

#[derive(Resource)]
struct RefractionPipeline {
    layout: BindGroupLayoutDescriptor,
    shader: Handle<Shader>,
    fullscreen: FullscreenShader,
    sampler: Sampler,
}
fn initialize(
    mut commands: Commands,
    assets: Res<AssetServer>,
    device: Res<RenderDevice>,
    fullscreen: Res<FullscreenShader>,
) {
    let entries = BindGroupLayoutEntries::sequential(
        ShaderStages::FRAGMENT,
        (
            texture_2d(TextureSampleType::Float { filterable: true }),
            sampler(SamplerBindingType::Filtering),
            texture_2d(TextureSampleType::Float { filterable: true }),
            uniform_buffer::<RefractionView>(false),
        ),
    );
    commands.insert_resource(RefractionPipeline {
        layout: BindGroupLayoutDescriptor::new("particle refraction", &entries),
        shader: assets.load("embedded://bevy_wc3/shaders/particle_refraction.wgsl"),
        fullscreen: fullscreen.clone(),
        sampler: device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        }),
    });
}
impl SpecializedRenderPipeline for RefractionPipeline {
    type Key = TextureFormat;
    fn specialize(&self, format: TextureFormat) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("particle refraction composition".into()),
            layout: vec![self.layout.clone()],
            vertex: self.fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

#[derive(Default)]
struct SourceBindings {
    // Ping-pong scene views alternate even when neither texture reallocates.
    groups: Vec<(TextureViewId, BindGroup)>,
}
#[derive(Component)]
pub(super) struct RefractionTarget {
    mask: CachedTexture,
    multisampled: Option<CachedTexture>,
    settings: UniformBuffer<RefractionView>,
    mask_binding: BindGroup,
    pipeline: CachedRenderPipelineId,
    source_bindings: Mutex<SourceBindings>,
}

pub(super) fn mask_view_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "particle refraction mask view",
        &BindGroupLayoutEntries::single(
            ShaderStages::FRAGMENT,
            uniform_buffer::<RefractionView>(false),
        ),
    )
}

pub(super) struct SetRefractionView;
impl<P: PhaseItem> RenderCommand<P> for SetRefractionView {
    type Param = ();
    type ViewQuery = Read<RefractionTarget>;
    type ItemQuery = ();
    fn render<'w>(
        _: &P,
        target: &'w RefractionTarget,
        _: Option<()>,
        _: SystemParamItem<'w, '_, ()>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(3, &target.mask_binding, &[]);
        RenderCommandResult::Success
    }
}

fn sort_phases(
    mut phases: ResMut<RefractionPhases>,
    views: Query<&ExtractedView>,
    mut live: Local<HashSet<RetainedViewEntity>>,
) {
    live.clear();
    live.extend(views.iter().map(|view| view.retained_view_entity));
    phases.0.retain(|key, _| live.contains(key));
    for view in &views {
        if let Some(phase) = phases.0.get_mut(&view.retained_view_entity) {
            phase.recalculate_sort_keys(view);
            phase.sort();
        }
    }
}

type RefractionViews<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ExtractedView,
        &'static ExtractedCamera,
        &'static ViewTarget,
        &'static ViewDepthTexture,
        Option<&'static MainPassResolutionOverride>,
        Option<&'static ParticleRefraction>,
        Option<&'static mut RefractionTarget>,
    ),
>;

#[allow(clippy::too_many_arguments)]
fn prepare_views(
    mut commands: Commands,
    phases: Res<RefractionPhases>,
    mut views: RefractionViews,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut textures: ResMut<TextureCache>,
    pipeline: Res<RefractionPipeline>,
    cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<RefractionPipeline>>,
) {
    for (entity, view, camera, target, depth, resolution, settings, previous) in &mut views {
        let active = phases
            .0
            .get(&view.retained_view_entity)
            .is_some_and(|phase| !phase.items.is_empty());
        if !active {
            if previous.is_some() {
                commands.entity(entity).remove::<RefractionTarget>();
            }
            continue;
        }
        let settings = settings.copied().unwrap_or_default();
        if !settings.strength.is_finite()
            || settings.strength < 0.
            || !settings.fade_distance.is_finite()
            || settings.fade_distance <= 0.
            || !settings.fade_distance.recip().is_finite()
        {
            error!("invalid particle refraction settings for view {entity:?}");
            commands.entity(entity).remove::<RefractionTarget>();
            continue;
        }
        if settings.strength == 0. {
            commands.entity(entity).remove::<RefractionTarget>();
            continue;
        }
        let size = depth.texture.size();
        let samples = depth.texture.sample_count();
        let descriptor = TextureDescriptor {
            label: Some("particle refraction mask"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let mask = textures.get(&device, descriptor.clone());
        let multisampled = (samples > 1).then(|| {
            textures.get(
                &device,
                TextureDescriptor {
                    sample_count: samples,
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    ..descriptor
                },
            )
        });
        let viewport_size =
            Viewport::from_viewport_and_override(camera.viewport.as_ref(), resolution)
                .map_or(view.viewport.zw(), |viewport| viewport.physical_size);
        let value = RefractionView {
            projection_scale: Vec2::new(
                view.clip_from_view.x_axis.x,
                -view.clip_from_view.y_axis.y,
            ) * viewport_size.as_vec2()
                / Vec2::new(size.width as f32, size.height as f32),
            strength: settings.strength,
            fade_distance_inverse: settings.fade_distance.recip(),
        };
        let specialized = pipelines.specialize(&cache, &pipeline, target.main_texture_format());
        if let Some(mut previous) = previous {
            if previous.mask.default_view.id() != mask.default_view.id() {
                previous.mask = mask;
                previous.source_bindings.lock().unwrap().groups.clear();
            }
            previous.multisampled = multisampled;
            previous.pipeline = specialized;
            if *previous.settings.get() != value {
                previous.settings.set(value);
                previous.settings.write_buffer(&device, &queue);
            }
            continue;
        }
        let mut uniform = UniformBuffer::from(value);
        uniform.write_buffer(&device, &queue);
        let mask_binding = device.create_bind_group(
            "particle refraction mask view",
            &cache.get_bind_group_layout(&mask_view_layout()),
            &BindGroupEntries::single(uniform.binding().unwrap()),
        );
        commands.entity(entity).insert(RefractionTarget {
            mask,
            multisampled,
            settings: uniform,
            mask_binding,
            pipeline: specialized,
            source_bindings: Mutex::default(),
        });
    }
}

type RefractionRenderView = (
    &'static ExtractedCamera,
    &'static ExtractedView,
    &'static ViewTarget,
    &'static ViewDepthTexture,
    &'static RefractionTarget,
    Option<&'static MainPassResolutionOverride>,
);
fn refract(
    world: &World,
    view: ViewQuery<RefractionRenderView>,
    phases: Res<RefractionPhases>,
    pipeline: Res<RefractionPipeline>,
    cache: Res<PipelineCache>,
    mut ctx: RenderContext,
) {
    let entity = view.entity();
    let (camera, extracted, target, depth, textures, resolution) = view.into_inner();
    let Some(phase) = phases.0.get(&extracted.retained_view_entity) else {
        return;
    };
    let Some(composition) = cache.get_render_pipeline(textures.pipeline) else {
        return;
    };
    if phase.items.is_empty() {
        return;
    }
    {
        let attachment = RenderPassColorAttachment {
            view: textures
                .multisampled
                .as_ref()
                .map_or(&textures.mask.default_view, |texture| &texture.default_view),
            resolve_target: textures
                .multisampled
                .as_ref()
                .map(|_| &*textures.mask.default_view),
            depth_slice: None,
            ops: Operations {
                load: LoadOp::Clear(default()),
                store: StoreOp::Store,
            },
        };
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("particle refraction mask"),
            color_attachments: &[Some(attachment)],
            depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if let Some(viewport) =
            Viewport::from_viewport_and_override(camera.viewport.as_ref(), resolution)
        {
            pass.set_camera_viewport(&viewport);
        }
        if let Err(error) = phase.render(&mut pass, world, entity) {
            error!("particle refraction mask: {error:?}");
        }
    }
    let post = target.post_process_write();
    let mut bindings = textures.source_bindings.lock().unwrap();
    let source = post.source.id();
    let index = if let Some(index) = bindings.groups.iter().position(|(id, _)| *id == source) {
        index
    } else {
        if bindings.groups.len() == 2 {
            bindings.groups.clear();
        }
        let group = ctx.render_device().create_bind_group(
            "particle refraction composition",
            &cache.get_bind_group_layout(&pipeline.layout),
            &BindGroupEntries::sequential((
                post.source,
                &pipeline.sampler,
                &textures.mask.default_view,
                textures.settings.binding().unwrap(),
            )),
        );
        bindings.groups.push((source, group));
        bindings.groups.len() - 1
    };
    let mut pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("particle refraction composition"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post.destination,
                resolve_target: None,
                depth_slice: None,
                ops: Operations {
                    load: LoadOp::Clear(default()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    pass.set_pipeline(composition);
    pass.set_bind_group(0, &bindings.groups[index].1, &[]);
    pass.draw(0..3, 0..1);
}
