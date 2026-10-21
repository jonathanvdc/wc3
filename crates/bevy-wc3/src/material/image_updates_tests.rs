use super::*;
use crate::Wc3BevyPlugin;
use bevy::app::PluginsState;
use bevy::asset::{uuid_handle, AssetApp, AssetPlugin, RenderAssetUsages};
use bevy::ecs::message::{MessageCursor, Messages};
use bevy::pbr::{
    material_uses_bindless_resources, MaterialBindGroupAllocators, RenderMaterialBindings,
};
use bevy::prelude::*;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{BindGroupId, Extent3d, TextureDimension, TextureFormat};
use bevy::render::renderer::RenderDevice;
use bevy::render::settings::{WgpuFeatures, WgpuSettings};
use bevy::render::texture::GpuImage;
use bevy::render::{RenderApp, RenderPlugin};
use bevy::winit::WinitPlugin;
use std::any::TypeId;
use std::thread::yield_now;
use std::time::{Duration, Instant};

#[test]
fn index_tracks_shared_owners_image_edits_and_material_removal() {
    let image: Handle<Image> = uuid_handle!("8336eeb8-0fbf-4b42-a143-b341be39ca11");
    let other: Handle<Image> = uuid_handle!("8336eeb8-0fbf-4b42-a143-b341be39ca12");
    let a: Handle<ModelMaterial> = uuid_handle!("8336eeb8-0fbf-4b42-a143-b341be39ca13");
    let b: Handle<ModelMaterial> = uuid_handle!("8336eeb8-0fbf-4b42-a143-b341be39ca14");
    let mut material = ModelMaterial::default();
    material.base.normal_map_texture = Some(image.clone());
    material.extension.secondary = Some(image.clone());
    let mut index = MaterialImages::default();
    index.update(a.id(), &material);
    index.update(b.id(), &material);
    assert_eq!(index.materials[&a.id()], [image.id()]);
    assert_eq!(index.owners[&image.id()].len(), 2);
    material.base.normal_map_texture = None;
    material.extension.secondary = Some(other.clone());
    index.update(a.id(), &material);
    assert_eq!(index.owners[&image.id()], HashSet::from([b.id()]));
    assert_eq!(index.owners[&other.id()], HashSet::from([a.id()]));
    index.remove(b.id());
    assert!(!index.owners.contains_key(&image.id()));
    index.remove(a.id());
    assert!(index.owners.is_empty());
    assert!(index.materials.is_empty());
}
fn modified(
    app: &App,
    cursor: &mut MessageCursor<AssetEvent<ModelMaterial>>,
) -> Vec<AssetId<ModelMaterial>> {
    cursor
        .read(
            app.world()
                .resource::<Messages<AssetEvent<ModelMaterial>>>(),
        )
        .filter_map(|event| {
            if let AssetEvent::Modified { id } = event {
                Some(*id)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn published_image_changes_notify_only_current_material_owners_once() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Image>().init_asset::<ModelMaterial>();
    register_material_image_updates(&mut app);
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    let unrelated = app
        .world_mut()
        .resource_mut::<Assets<ModelMaterial>>()
        .add(ModelMaterial::default());
    let mut material = ModelMaterial::default();
    material.base.base_color_texture = Some(image.clone());
    material.extension.fourth = Some(image.clone());
    let owner = app
        .world_mut()
        .resource_mut::<Assets<ModelMaterial>>()
        .add(material);
    app.update();
    let mut cursor = MessageCursor::default();
    modified(&app, &mut cursor);
    app.world_mut()
        .write_message(AssetEvent::Modified { id: image.id() });
    app.world_mut()
        .write_message(AssetEvent::Modified { id: image.id() });
    app.update();
    assert_eq!(modified(&app, &mut cursor), [owner.id()]);
    app.update();
    assert!(
        modified(&app, &mut cursor).is_empty(),
        "synthetic edits must not feed back indefinitely"
    );
    app.world_mut()
        .resource_mut::<Assets<ModelMaterial>>()
        .get_mut(owner.id())
        .unwrap()
        .base
        .base_color_texture = None;
    app.world_mut()
        .resource_mut::<Assets<ModelMaterial>>()
        .get_mut(owner.id())
        .unwrap()
        .extension
        .fourth = None;
    app.world_mut()
        .write_message(AssetEvent::Removed { id: image.id() });
    app.update();
    let events = modified(&app, &mut cursor);
    assert_eq!(events.len(), 2, "only the two ordinary user edits remain");
    assert!(events
        .iter()
        .all(|id| *id == owner.id() && *id != unrelated.id()));
    assert!(!app
        .world()
        .resource::<MaterialImages>()
        .owners
        .contains_key(&image.id()));
}
fn image(color: u8) -> Image {
    Image::new_fill(
        Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[color, 127, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
fn prepared_group(app: &App, material: AssetId<ModelMaterial>) -> Option<BindGroupId> {
    let world = app.sub_app(RenderApp).world();
    let binding = world
        .get_resource::<RenderMaterialBindings>()?
        .get(&material.untyped())?;
    let allocator = world
        .get_resource::<MaterialBindGroupAllocators>()?
        .get(&TypeId::of::<ModelMaterial>())?;
    Some(allocator.get(binding.group)?.bind_group()?.id())
}
#[test]
#[ignore = "requires a host GPU; run explicitly with --ignored --exact"]
fn same_handle_image_replacement_reprepares_real_material_allocator() {
    image_replacement_reprepares_allocator(false);
}
#[test]
#[ignore = "requires a host GPU; run explicitly with --ignored --exact"]
fn same_handle_image_replacement_reprepares_non_bindless_material_allocator() {
    image_replacement_reprepares_allocator(true);
}
fn image_replacement_reprepares_allocator(no_bindless: bool) {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(RenderPlugin {
                render_creation: WgpuSettings {
                    disabled_features: no_bindless.then_some(WgpuFeatures::TEXTURE_BINDING_ARRAY),
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                ..default()
            })
            .disable::<WinitPlugin>()
            .disable::<PipelinedRenderingPlugin>(),
    );
    app.add_plugins(Wc3BevyPlugin);
    let deadline = Instant::now() + Duration::from_secs(60);
    while app.plugins_state() != PluginsState::Ready {
        assert!(Instant::now() < deadline);
        yield_now();
    }
    app.finish();
    app.cleanup();
    let device = app.sub_app(RenderApp).world().resource::<RenderDevice>();
    let uses_bindless = material_uses_bindless_resources::<ModelMaterial>(device);
    eprintln!(
        "ModelMaterial allocator: disable_bindless={no_bindless}, actual_bindless={uses_bindless}"
    );
    if no_bindless {
        assert!(!device
            .features()
            .contains(WgpuFeatures::TEXTURE_BINDING_ARRAY));
        assert!(
            !uses_bindless,
            "explicitly disabled texture arrays must select the ordinary allocator"
        );
    }
    let images: Vec<_> = (0..8)
        .map(|_| {
            app.world_mut()
                .resource_mut::<Assets<Image>>()
                .add(image(32))
        })
        .collect();
    let mut material = ModelMaterial::default();
    material.base.base_color_texture = Some(images[0].clone());
    material.base.normal_map_texture = Some(images[1].clone());
    let extension = &mut material.extension;
    for (slot, image) in [
        &mut extension.secondary,
        &mut extension.orm,
        &mut extension.team_color,
        &mut extension.environment,
        &mut extension.tertiary,
        &mut extension.fourth,
    ]
    .into_iter()
    .zip(&images[2..])
    {
        *slot = Some(image.clone());
    }
    let handle = app
        .world_mut()
        .resource_mut::<Assets<ModelMaterial>>()
        .add(material);
    while prepared_group(&app, handle.id()).is_none() {
        assert!(
            Instant::now() < deadline,
            "material allocator preparation timed out"
        );
        app.update();
        yield_now();
    }
    for _ in 0..4 {
        app.update();
    }
    let mut image_events = MessageCursor::<AssetEvent<Image>>::default();
    let mut material_events = MessageCursor::<AssetEvent<ModelMaterial>>::default();
    for (slot, image_handle) in images.into_iter().enumerate() {
        // Exclude prior slots and stability frames from this replacement's evidence.
        image_events
            .read(app.world().resource::<Messages<AssetEvent<Image>>>())
            .count();
        material_events
            .read(
                app.world()
                    .resource::<Messages<AssetEvent<ModelMaterial>>>(),
            )
            .count();
        let mut image_changes = 0;
        let mut material_changes = 0;
        let before = prepared_group(&app, handle.id()).unwrap();
        let old_view = app
            .sub_app(RenderApp)
            .world()
            .resource::<RenderAssets<GpuImage>>()
            .get(image_handle.id())
            .unwrap()
            .texture_view
            .id();
        // Bevy deliberately reuses same-descriptor COPY_DST textures/views for
        // pixel edits. Resize under the same asset id to exercise replacement.
        let mut replacement = image(192);
        replacement.resize(Extent3d {
            width: 3,
            height: 2,
            depth_or_array_layers: 1,
        });
        *app.world_mut()
            .resource_mut::<Assets<Image>>()
            .get_mut(image_handle.id())
            .unwrap() = replacement;
        let mut frame = 0;
        loop {
            app.update();
            yield_now();
            let world = app.sub_app(RenderApp).world();
            let new_view = world
                .resource::<RenderAssets<GpuImage>>()
                .get(image_handle.id())
                .unwrap()
                .texture_view
                .id();
            let group = prepared_group(&app, handle.id());
            let view_changed = new_view != old_view;
            let group_changed = group.is_some_and(|group| group != before);
            image_changes += image_events
                .read(app.world().resource::<Messages<AssetEvent<Image>>>())
                .filter(|event| matches!(event, AssetEvent::Modified { id } if *id == image_handle.id()))
                .count();
            material_changes += material_events
                .read(
                    app.world()
                        .resource::<Messages<AssetEvent<ModelMaterial>>>(),
                )
                .filter(|event| matches!(event, AssetEvent::Modified { id } if *id == handle.id()))
                .count();
            assert!(Instant::now() < deadline,
                "same-id image reload slot={slot} timed out: view_changed={view_changed}, group_changed={group_changed}, current_group={group:?}, image_modified={image_changes}, material_modified={material_changes}");
            if view_changed && group_changed {
                assert!(
                    image_changes > 0,
                    "slot={slot}: replacement must publish its Image Modified event"
                );
                assert!(
                    material_changes > 0,
                    "slot={slot}: image change must invalidate its material"
                );
                eprintln!("image reload slot={slot} frames={} image_modified={image_changes} material_modified={material_changes} view_replaced=true allocator_rebound=true", frame + 1);
                break;
            }
            frame += 1;
        }
        let stable = prepared_group(&app, handle.id());
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(
            prepared_group(&app, handle.id()),
            stable,
            "no repeated reallocation after one image edit"
        );
    }
}

#[test]
fn empty_preparation_set_preserves_transitive_system_order() {
    #[derive(Resource, Default)]
    struct Order(Vec<u8>);
    fn prepare_image(mut order: ResMut<Order>) {
        order.0.push(1);
    }
    fn prepare_material(mut order: ResMut<Order>) {
        order.0.push(2);
    }
    let mut world = World::new();
    world.init_resource::<Order>();
    let mut schedule = Schedule::default();
    schedule.add_systems((prepare_material, prepare_image));
    schedule.configure_sets(
        ImageMaterialPreparation
            .after(prepare_image)
            .before(prepare_material),
    );
    schedule.run(&mut world);
    assert_eq!(world.resource::<Order>().0, [1, 2]);
}
