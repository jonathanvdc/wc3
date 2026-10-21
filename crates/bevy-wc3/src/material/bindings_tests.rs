use super::*;
use bevy::app::PluginsState;
use bevy::asset::uuid_handle;
use bevy::ecs::system::SystemState;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::render::RenderStartup;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::render_resource::{BindGroupEntry, BufferId};
use bevy::winit::WinitPlugin;
use std::thread::yield_now;
use std::time::{Duration, Instant};

#[test]
fn matrix_encoding_preserves_gpu_column_stride_and_zero_padding() {
    let material = CombinerUniform {
        uv_transform: Mat3::from_cols_array(&[1., 2., 3., 4., 5., 6., 7., 8., 9.]),
        ..Default::default()
    };
    let bytes = encode_uniform(&material);
    for (index, offset) in [0, 4, 8, 16, 20, 24, 32, 36, 40].into_iter().enumerate() {
        assert_eq!(
            &bytes[offset..offset + 4],
            &(index as f32 + 1.).to_le_bytes()
        );
    }
    for offset in [12, 28, 44] {
        assert_eq!(&bytes[offset..offset + 4], &[0; 4]);
    }
    assert_eq!(bytes.len() as u64, CombinerUniform::min_size().get());
}
#[test]
fn encoding_keys_preserve_float_bits_instead_of_numeric_equality() {
    let mut material = WorldUniform::default();
    let zero = encode_uniform(&material);
    material.ambient.x = -0.;
    assert_ne!(encode_uniform(&material), zero);
    material.ambient.x = f32::from_bits(0x7fc0_0001);
    let first_nan = encode_uniform(&material);
    material.ambient.x = f32::from_bits(0x7fc0_0002);
    assert_ne!(encode_uniform(&material), first_nan);
}

fn gpu_app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                ..Default::default()
            })
            .disable::<WinitPlugin>()
            .disable::<PipelinedRenderingPlugin>(),
    );
    register_material_uniform_buffers(&mut app);
    let deadline = Instant::now() + Duration::from_secs(60);
    while app.plugins_state() != PluginsState::Ready {
        assert!(
            Instant::now() < deadline,
            "GPU plugin initialization timed out"
        );
        yield_now();
    }
    app.finish();
    app.cleanup();
    app.update();
    app
}
fn prepare(
    world: &mut World,
    material: &ModelMaterialExtension,
) -> Result<UnpreparedBindGroup, AsBindGroupError> {
    let device = world.resource::<RenderDevice>().clone();
    let entries = ModelMaterialExtension::bind_group_layout_entries(&device, true);
    let layout = device.create_bind_group_layout("material cache test", &entries);
    let mut params = SystemState::<<ModelMaterialExtension as AsBindGroup>::Param>::new(world);
    let group = material.unprepared_bind_group(
        &layout,
        &device,
        &mut params.get_mut(world).unwrap(),
        true,
    )?;
    let entries: Vec<_> = group
        .bindings
        .iter()
        .map(|(binding, resource)| BindGroupEntry {
            binding: *binding,
            resource: resource.get_binding(),
        })
        .collect();
    // Exercise real GPU layout/resource validation, not only identity bookkeeping.
    device.create_bind_group("material cache validation", &layout, &entries);
    Ok(group)
}
fn uniform_ids(group: &UnpreparedBindGroup) -> [BufferId; 3] {
    [100, 103, 110].map(|binding| {
        let (_, resource) = group
            .bindings
            .iter()
            .find(|(index, _)| *index == binding)
            .unwrap();
        let OwnedBindingResource::Buffer(buffer) = resource else {
            panic!("uniform buffer")
        };
        buffer.id()
    })
}
#[test]
#[ignore = "requires a host GPU; run explicitly with --ignored --exact"]
fn material_uniform_resources_share_evict_reset_and_retry_without_changing_images() {
    let mut app = gpu_app();
    let world = app.sub_app_mut(RenderApp).world_mut();
    let mut material = ModelMaterialExtension::default();
    let first = prepare(world, &material).unwrap();
    let equal = prepare(world, &material).unwrap();
    let original = uniform_ids(&first);
    assert_eq!(uniform_ids(&equal), original);
    // A base-only opacity edit leaves exactly this same extension snapshot.
    let mut combined = ExtendedMaterial {
        base: StandardMaterial::default(),
        extension: material.clone(),
    };
    combined.base.base_color = Color::srgba(1., 1., 1., 0.25);
    assert_eq!(
        uniform_ids(&prepare(world, &combined.extension).unwrap()),
        original
    );
    material.combiner.weights.x = 0.5;
    let changed = uniform_ids(&prepare(world, &material).unwrap());
    assert_ne!(changed[0], original[0]);
    assert_eq!(&changed[1..], &original[1..]);
    let image_ids = |group: &UnpreparedBindGroup| {
        group
            .bindings
            .iter()
            .filter_map(|(binding, resource)| {
                if let OwnedBindingResource::TextureView(_, view) = resource {
                    Some((*binding, view.id()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(image_ids(&first), image_ids(&equal));
    // Many distinct combiner states cannot evict default Hd/World uniforms.
    for index in 0..=UNIFORMS_PER_TYPE {
        material.combiner.weights.x = index as f32 + 2.;
        prepare(world, &material).unwrap();
    }
    let cache = world.resource::<MaterialUniformBuffers>();
    assert_eq!(cache.combiner.entries.len(), UNIFORMS_PER_TYPE);
    assert_eq!(cache.hd.entries.len(), 1);
    assert_eq!(cache.world.entries.len(), 1);
    assert_eq!(
        uniform_ids(&first),
        original,
        "eviction preserves retained owned buffers"
    );
    world.run_schedule(RenderStartup);
    let cache = world.resource::<MaterialUniformBuffers>();
    assert!(cache.combiner.entries.is_empty());
    material = ModelMaterialExtension::default();
    let reset = uniform_ids(&prepare(world, &material).unwrap());
    assert!(reset.into_iter().zip(original).all(|(a, b)| a != b));
    // No image resolve means no new uniforms retained by the retry path.
    world.insert_resource(MaterialUniformBuffers::default());
    material.secondary = Some(uuid_handle!("ff8cc27f-5644-48f7-811a-24f24be839ee"));
    assert!(matches!(
        prepare(world, &material),
        Err(AsBindGroupError::RetryNextUpdate)
    ));
    assert!(
        world
            .resource::<MaterialUniformBuffers>()
            .combiner
            .entries
            .is_empty()
    );
    let handle = material.secondary.clone().unwrap();
    let image = world.resource::<FallbackImage>().d2.clone();
    world
        .resource_mut::<RenderAssets<GpuImage>>()
        .insert(handle.id(), image.clone());
    let before_reload = prepare(world, &material).unwrap();
    let mut replacement = image;
    replacement.texture_view = replacement.texture.create_view(&Default::default());
    replacement.sampler = world
        .resource::<RenderDevice>()
        .create_sampler(&Default::default());
    world
        .resource_mut::<RenderAssets<GpuImage>>()
        .insert(handle.id(), replacement);
    let after_reload = prepare(world, &material).unwrap();
    assert_eq!(uniform_ids(&before_reload), uniform_ids(&after_reload));
    assert_ne!(
        image_ids(&before_reload),
        image_ids(&after_reload),
        "uniform reuse never pins old image views"
    );
    // Optional acceleration preserves standalone material preparation.
    world.remove_resource::<MaterialUniformBuffers>();
    material.secondary = None;
    let uncached_a = uniform_ids(&prepare(world, &material).unwrap());
    let uncached_b = uniform_ids(&prepare(world, &material).unwrap());
    assert!(uncached_a.into_iter().zip(uncached_b).all(|(a, b)| a != b));
}

#[test]
#[ignore = "requires a host GPU; run explicitly with --ignored --exact"]
fn bindless_snapshot_emits_plain_data_without_uniform_buffer_arrays() {
    let mut app = gpu_app();
    let world = app.sub_app_mut(RenderApp).world_mut();
    let device = world.resource::<RenderDevice>().clone();
    if !ModelMaterialExtension::bindless_supported(&device) {
        return;
    }
    let entries = ModelMaterialExtension::bind_group_layout_entries(&device, false);
    let layout = device.create_bind_group_layout("bindless data test", &entries);
    let mut params = SystemState::<<ModelMaterialExtension as AsBindGroup>::Param>::new(world);
    let material = ModelMaterialExtension::default();
    let group = material
        .unprepared_bind_group(&layout, &device, &mut params.get_mut(world).unwrap(), false)
        .unwrap();
    assert_eq!(group.bindings.len(), 13);
    assert!(
        group.bindings.iter().any(
            |(slot, resource)| *slot == 50 && matches!(resource, OwnedBindingResource::Data(_))
        )
    );
    assert!(
        !group
            .bindings
            .iter()
            .any(|(_, resource)| matches!(resource, OwnedBindingResource::Buffer(_)))
    );
    let cached_entries = ModelMaterialExtension::bind_group_layout_entries(&device, true);
    assert_eq!(
        cached_entries
            .iter()
            .map(|entry| entry.binding)
            .collect::<Vec<_>>(),
        [
            100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 114, 115
        ]
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry.binding == 116 && entry.count.is_none())
    );
    let data_before = group.bindings.iter().find(|(slot, _)| *slot == 50).unwrap();
    let OwnedBindingResource::Data(before) = &data_before.1 else {
        panic!("plain data")
    };
    let mut edited = material;
    edited.combiner.weights.y = 0.125;
    let changed = edited
        .unprepared_bind_group(&layout, &device, &mut params.get_mut(world).unwrap(), false)
        .unwrap();
    let data_after = changed
        .bindings
        .iter()
        .find(|(slot, _)| *slot == 50)
        .unwrap();
    let OwnedBindingResource::Data(after) = &data_after.1 else {
        panic!("plain data")
    };
    assert_ne!(
        before.0, after.0,
        "ordinary asset edits change the row published to Bevy"
    );
    edited.secondary = Some(uuid_handle!("ff8cc27f-5644-48f7-811a-24f24be839ee"));
    assert!(matches!(
        edited.unprepared_bind_group(&layout, &device, &mut params.get_mut(world).unwrap(), false),
        Err(AsBindGroupError::RetryNextUpdate)
    ));
    // Bevy's material allocator, not this direct helper, creates final array bindgroups.
}
