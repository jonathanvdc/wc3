use std::borrow::Cow;
use wc3::model::common::IntoOwned;
use wc3::model::materials::bindings::{Coordinates, ImageSelection, TextureBinding};
use wc3::model::materials::program::{MaterialProgram, MaterialStep, MaterialSurface};
use wc3::model::materials::state::{
    BlendEquation, BlendOperations, DepthState, PassState, Rasterization,
};
use wc3::model::materials::{m2, mdlx, wmo};
use wc3::model::profiles::Universal;
use wc3::model::{
    AddressMode, Construct, ImageResource, Model, ModelMetadata, PropertyValues, Sampler, Texture,
    Vec3,
};

fn assert_static<T: 'static>(_: &T) {}

#[test]
fn generic_material_components_detach_hidden_borrows_in_every_domain() {
    let name = "custom input vocabulary".to_owned();
    let bytes = vec![1u8, 2, 3, 4];
    let owned = vec![5u8, 6, 7];
    let allocation = owned.as_ptr();
    let step = TextureBinding {
        image: ImageSelection {
            source: Cow::Borrowed(name.as_str()),
            texture: None,
        },
        coordinates: Coordinates {
            source: Cow::Borrowed(bytes.as_slice()),
            transform: Cow::<[u8]>::Owned(owned),
        },
        sampling: PassState {
            blend:
                BlendOperations::<Cow<'_, str>, Cow<'_, str>, Cow<'_, str>, Cow<'_, str>>::Unknown(
                    Cow::Borrowed(name.as_str()),
                ),
            rasterization: Rasterization {
                depth: DepthState {
                    test: Cow::Borrowed(bytes.as_slice()),
                    write: Cow::Borrowed(bytes.as_slice()),
                    bias: Cow::Borrowed(name.as_str()),
                },
                faces: Cow::Borrowed(name.as_str()),
                lighting: BlendEquation {
                    enabled: Cow::Borrowed(name.as_str()),
                    operation: Cow::Borrowed(name.as_str()),
                    source: Cow::Borrowed(bytes.as_slice()),
                    destination: Cow::Borrowed(bytes.as_slice()),
                    constant: Cow::Borrowed(name.as_str()),
                },
                controls: Cow::Borrowed(bytes.as_slice()),
            },
            ordering: Cow::Borrowed(name.as_str()),
        },
        contribution: Cow::Borrowed(bytes.as_slice()),
    };
    let owned = MaterialProgram {
        declaration: Cow::Borrowed(name.as_str()),
        steps: vec![step],
    }
    .into_owned();
    assert_static(&owned);
    drop(name);
    drop(bytes);

    assert_eq!(owned.declaration, "custom input vocabulary");
    let input = &owned.steps[0];
    assert_eq!(input.coordinates.source.as_ref(), [1, 2, 3, 4]);
    assert_eq!(input.coordinates.transform.as_ref(), [5, 6, 7]);
    assert_eq!(input.coordinates.transform.as_ptr(), allocation);
    assert_eq!(input.image.source, "custom input vocabulary");
    assert_eq!(input.contribution.as_ref(), [1, 2, 3, 4]);
    assert!(matches!(
        input.sampling.blend,
        BlendOperations::Unknown(Cow::Owned(_))
    ));
    let rasterization = &input.sampling.rasterization;
    assert_eq!(rasterization.depth.test.as_ref(), [1, 2, 3, 4]);
    assert_eq!(rasterization.depth.write.as_ref(), [1, 2, 3, 4]);
    assert_eq!(rasterization.depth.bias, "custom input vocabulary");
    assert_eq!(rasterization.faces, "custom input vocabulary");
    assert_eq!(rasterization.controls.as_ref(), [1, 2, 3, 4]);
    assert_eq!(rasterization.lighting.enabled, "custom input vocabulary");
    assert_eq!(rasterization.lighting.operation, "custom input vocabulary");
    assert_eq!(rasterization.lighting.source.as_ref(), [1, 2, 3, 4]);
    assert_eq!(rasterization.lighting.destination.as_ref(), [1, 2, 3, 4]);
    assert_eq!(rasterization.lighting.constant, "custom input vocabulary");
    assert_eq!(input.sampling.ordering, "custom input vocabulary");
}

#[test]
fn universal_material_detachment_keeps_shader_variants_ids_and_owned_declarations() {
    let m2_name = "guild surface".to_owned();
    let wmo_name = "world surface".to_owned();
    let shader_name = "Shader_HD_Crystal".to_owned();
    let allocation = shader_name.as_ptr();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let image = builder.add_image_resource(ImageResource::file_data_id(42));
    let texture = Texture::new(image);
    let mut values = PropertyValues::default();

    let mut crystal = mdlx::MaterialStep::construct(&mut values);
    crystal.shader = mdlx::Shader::ReforgedCrystal(mdlx::ReforgedInputs::construct(&mut values));

    let mut declaration = m2::MaterialDeclaration::construct(&mut values);
    declaration.label = Cow::Borrowed(&m2_name);
    let color = declaration.color;
    let opacity = declaration.opacity;
    let mut input = m2::TextureUnit::construct(&mut values);
    input.image.texture = Some(texture);
    let coordinates = input.coordinates.clone();
    let palette = m2::GuildPalette {
        background: Vec3::new(0.1, 0.2, 0.3),
        emblem: Vec3::new(0.4, 0.5, 0.6),
        border: Vec3::new(0.7, 0.8, 0.9),
    };
    let guild = m2::MaterialStep::from_native(0x8000 | 28, declaration, vec![input; 3], palette);
    assert!(matches!(guild, m2::MaterialStep::OpaqueGuild(_)));

    let sampler = Sampler {
        u: AddressMode::Mirror,
        v: AddressMode::Clamp,
    };
    let world = wmo::MaterialStep::from_native(
        Some(20),
        wmo::MaterialDeclaration {
            name: Cow::Borrowed(&wmo_name),
            ground_type: 0xfeed,
            shader_overrides: Some(wmo::ShaderOverrides {
                color: [10, 20, 30, 40],
                texture_override: 0x1234_5678_90ab_cdef,
                control: 0xfedc_ba98,
            }),
            ..Default::default()
        },
        vec![
            TextureBinding {
                image: Some(image),
                coordinates: (),
                sampling: sampler,
                contribution: (),
            };
            4
        ],
    );
    let surface = MaterialSurface {
        declaration: Some(mdlx::MaterialDeclaration {
            shader_name: Cow::Owned(shader_name),
            priority_plane: -7,
            ..Default::default()
        }),
        steps: vec![
            MaterialStep::Mdlx(Box::new(crystal)),
            MaterialStep::M2(Box::new(guild)),
            MaterialStep::Wmo(Box::new(world)),
        ],
    };
    let expected = format!("{surface:?}");
    let owned: MaterialSurface<'static> = surface.into_owned();
    drop(m2_name);
    drop(wmo_name);

    assert_eq!(format!("{owned:?}"), expected);
    assert_eq!(
        owned.declaration.as_ref().unwrap().shader_name.as_ptr(),
        allocation
    );
    assert_eq!(owned.declaration.as_ref().unwrap().priority_plane, -7);
    let MaterialStep::Mdlx(crystal) = &owned.steps[0] else {
        panic!("MDLX step changed dialect");
    };
    assert!(matches!(crystal.shader, mdlx::Shader::ReforgedCrystal(_)));
    let MaterialStep::M2(guild) = &owned.steps[1] else {
        panic!("M2 step changed dialect");
    };
    assert!(matches!(**guild, m2::MaterialStep::OpaqueGuild(_)));
    assert!(matches!(guild.declaration().label, Cow::Owned(_)));
    assert_eq!(guild.declaration().color, color);
    assert_eq!(guild.declaration().opacity, opacity);
    assert_eq!(guild.guild_palette(), Some(palette));
    for input in guild.inputs() {
        assert_eq!(input.image.texture, Some(texture));
        assert_eq!(input.coordinates, coordinates);
    }
    let MaterialStep::Wmo(world) = &owned.steps[2] else {
        panic!("WMO step changed dialect");
    };
    assert!(matches!(
        **world,
        wmo::MaterialStep::TextureMaskedLayeredModulation(_)
    ));
    assert!(matches!(world.declaration().name, Cow::Owned(_)));
    assert_eq!(world.native_shader(), Some(20));
    assert_eq!(world.inputs().len(), 4);
    for input in world.inputs() {
        assert_eq!(input.image, Some(image));
        assert_eq!(input.sampling, sampler);
    }
}
