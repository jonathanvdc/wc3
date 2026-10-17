use wc3::model::controls::playback::ControllerProgram;
use wc3::model::materials::bindings::{Coordinates, ImageSelection};
use wc3::model::materials::m2::FrameSampling;
use wc3::model::materials::m2::{
    GuildMaterialStep, MaterialStep as M2MaterialStep, ShaderInput, TextureUnit,
};
use wc3::model::materials::mdlx::{LayeredSurface, SurfaceLayer};
use wc3::model::profiles::{m2, mdlx, SurfacePayload};
use wc3::model::{Construct as SlotConstruct, Values as SlotValues};
use wc3::model::{Error, Link, PropertyPayload, TextureTransform, ValidateLinks};

#[derive(Clone, Debug, Default)]
struct CheckedPass {
    depth: f32,
}
impl ValidateLinks for CheckedPass {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl SurfaceLayer for CheckedPass {
    fn validate(&self) -> Result<(), Error> {
        if self.depth.is_finite() {
            Ok(())
        } else {
            Err(Error::new("checked_pass.depth", "depth must be finite"))
        }
    }
}
impl ShaderInput for CheckedPass {
    fn validate(&self) -> Result<(), Error> {
        SurfaceLayer::validate(self)
    }
}

#[test]
fn surfaces_admit_independently_defined_pass_and_input_vocabularies() {
    let mut layered = LayeredSurface::<CheckedPass>::default();
    layered.layers.push(CheckedPass { depth: f32::NAN });
    assert!(layered.validate().is_err());
    layered.layers[0].depth = 0.5;
    layered.validate().unwrap();

    let mut input = CheckedPass { depth: f32::NAN };
    assert!(ShaderInput::validate(&input).is_err());
    input.depth = 1.;
    ShaderInput::validate(&input).unwrap();
}

#[test]
fn built_in_profiles_select_the_shared_owning_types() {
    let mut slot_values = SlotValues::default();
    let layered = LayeredSurface::default();
    let _: mdlx::Material<'_> = layered;
    let shader = M2MaterialStep::construct(&mut slot_values);
    let _: m2::Material<'_> = shader.into();
    let transform = TextureTransform::construct(&mut slot_values);
    let _: m2::TextureTransform = transform;
}

#[test]
fn controller_program_traverses_its_selected_block_vocabulary() {
    #[derive(Clone, Debug)]
    struct InvalidBlock;
    impl ValidateLinks for InvalidBlock {
        fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
            visitor(Link::InvalidValue);
        }
    }
    let program = ControllerProgram::<InvalidBlock, ()> {
        state_properties: vec![12],
        blocks: vec![InvalidBlock],
        unbound: vec![()],
    };
    let mut invalid = 0;
    program.visit_links(&mut |link| {
        if matches!(link, Link::InvalidValue) {
            invalid += 1;
        }
    });
    assert_eq!(invalid, 1);
}

#[test]
fn selected_coordinate_form_controls_which_destinations_exist() {
    let mut slot_values = SlotValues::default();
    use m2::properties::{Target, TextureUnitField};
    let input = TextureUnit {
        image: ImageSelection {
            source: SlotConstruct::construct(&mut slot_values),
            texture: None,
        },
        coordinates: Coordinates {
            source: 0,
            transform: (),
        },
        contribution: slot_values.add(1.),
        sampling: FrameSampling {
            frame: slot_values.add(0),
        },
    };
    let weight = Target::TextureUnit {
        step: 0,
        index: 0,
        field: TextureUnitField::Weight,
    }
    .identity()
    .to_owned();
    let translation = Target::TextureUnit {
        step: 0,
        index: 0,
        field: TextureUnitField::Translation,
    }
    .identity()
    .to_owned();
    assert!(input.supports_property(&weight, 0));
    assert!(!input.supports_property(&weight, 1));
    assert!(!input.supports_property(&translation, 0));
    let animated_input = TextureUnit {
        coordinates: Coordinates {
            source: input.coordinates.source,
            transform: TextureTransform::construct(&mut slot_values),
        },
        image: input.image,
        contribution: input.contribution,
        sampling: input.sampling,
    };
    assert!(animated_input.supports_property(&translation, 0));
}

#[test]
fn texture_bindings_traverse_every_selected_component() {
    use wc3::model::materials::bindings::TextureBinding;

    #[derive(Clone, Debug)]
    struct Component;
    impl ValidateLinks for Component {
        fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
            visitor(Link::InvalidValue);
        }
    }
    let binding = TextureBinding {
        image: Component,
        coordinates: Coordinates {
            source: Component,
            transform: Component,
        },
        sampling: Component,
        contribution: Component,
    };
    let mut count = 0;
    binding.visit_links(&mut |link| {
        if matches!(link, Link::InvalidValue) {
            count += 1;
        }
    });
    assert_eq!(count, 5);
}

#[test]
fn m3_input_validation_reaches_nested_components_at_publication() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::m3::{
        MaterialStep as M3MaterialStep, SamplingProgram, StandardMaterialStep, TextureLayer,
    };
    use wc3::model::profiles::M3;
    use wc3::model::{Material, Model};

    let mut layer = TextureLayer::construct(&mut slot_values);
    layer.binding.sampling = SamplingProgram {
        noise_amplitude: f32::NAN,
        ..SlotConstruct::construct(&mut slot_values)
    };
    let standard = StandardMaterialStep {
        diffuse: Some(layer),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut builder = Model::<M3>::builder("invalid input");
    builder
        .add_material(Material {
            name: "surface".into(),
            surface: M3MaterialStep::Standard(Box::new(standard)),
        })
        .unwrap();
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());
}

#[test]
fn model_rasterization_preserves_unknown_flags_and_edits_known_operations() {
    use wc3::model::materials::m2::{material_flags, rasterization};
    use wc3::model::materials::state::Operation;
    use wc3::model::values::model::MaterialFlags;

    for raw in [0, 0x001f, 0x8000, 0xffff] {
        let flags = MaterialFlags::from_bits_retain(raw);
        let mut state = rasterization(flags);
        assert_eq!(material_flags(&state), flags);
        state.depth.write = Operation::Disabled;
        let edited = material_flags(&state);
        assert!(edited.no_depth_write());
        assert_eq!(edited.bits() & !0x0010, raw & !0x0010);
    }
}

#[test]
fn image_source_vocabularies_can_supply_graph_relationships() {
    use wc3::model::materials::bindings::TextureBinding;
    use wc3::model::profiles::Universal;
    use wc3::model::Model;

    let mut resources = Model::<Universal>::builder("textures");
    let source = resources.add_file_texture("generated-source.blp").unwrap();
    let binding = TextureBinding {
        image: ImageSelection {
            source,
            texture: None,
        },
        coordinates: (),
        sampling: (),
        contribution: (),
    };
    let mut found = false;
    binding.visit_links(&mut |link| {
        if let Link::SampledImage(id) = link {
            found |= id == source;
        }
    });
    assert!(found);
}

#[test]
fn guild_palette_defaults_are_neutral_and_nonfinite_settings_are_rejected() {
    let mut slot_values = SlotValues::default();
    use wc3::model::materials::m2::GuildPalette;
    let mut shader =
        M2MaterialStep::Guild(Box::new(GuildMaterialStep::construct(&mut slot_values)));
    assert_eq!(
        shader.guild_palette().unwrap(),
        GuildPalette {
            background: [1.; 3],
            emblem: [1.; 3],
            border: [1.; 3],
        }
    );
    if let M2MaterialStep::Guild(v) = &mut shader {
        v.palette.emblem[1] = f32::NAN;
    }
    assert!(shader.validate().is_err());
    if let M2MaterialStep::Guild(v) = &mut shader {
        v.palette.emblem[1] = 0.5;
    }
    shader.validate().unwrap();
}
