use wc3::model::semantic::controls::playback::ControllerProgram;
use wc3::model::semantic::materials::bindings::{Coordinates, ImageSelection};
use wc3::model::semantic::materials::layered::{LayeredSurface, SurfaceLayer};
use wc3::model::semantic::materials::model_shader::FrameSampling;
use wc3::model::semantic::materials::model_shader::{ShaderInput, ShaderPass, TextureUnit};
use wc3::model::semantic::profiles::{m2, mdlx, SurfacePayload};
use wc3::model::semantic::{
    Animated, Error, Link, PropertyPayload, TextureTransform, ValidateLinks,
};

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

    let mut shader = ShaderPass::<CheckedPass>::default();
    shader.textures.push(CheckedPass { depth: f32::NAN });
    assert!(shader.validate().is_err());
    shader.textures[0].depth = 1.;
    shader.validate().unwrap();
}

#[test]
fn built_in_profiles_select_the_shared_owning_types() {
    let layered = LayeredSurface::default();
    let _: mdlx::Material<'_> = layered;
    let shader = ShaderPass::default();
    let _: m2::Material<'_> = shader;
    let transform = TextureTransform::default();
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
    use m2::properties::{Target, TextureUnitField};
    let input = TextureUnit {
        image: ImageSelection {
            source: Default::default(),
            texture: None,
        },
        coordinates: Coordinates {
            source: 0,
            transform: (),
        },
        contribution: Animated::constant(1.),
        sampling: FrameSampling {
            frame: Animated::constant(0),
        },
    };
    let weight = Target::TextureUnit {
        index: 0,
        field: TextureUnitField::Weight,
    }
    .identity()
    .to_owned();
    let translation = Target::TextureUnit {
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
            transform: TextureTransform::default(),
        },
        image: input.image,
        contribution: input.contribution,
        sampling: input.sampling,
    };
    assert!(animated_input.supports_property(&translation, 0));
}

#[test]
fn broad_surface_selection_preserves_property_and_validation_policies() {
    use wc3::model::semantic::profiles::Unavailable;
    use wc3::model::semantic::{MaterialSurface, PbrSurface, PropertyPayload, SurfaceSet};

    let color = m2::properties::Target::Color.identity().to_owned();
    let pbr = MaterialSurface::Generic(Box::new(PbrSurface::default()));
    assert!(!pbr.supports_property(&color));

    type CustomSurfaces = SurfaceSet<
        Unavailable,
        Unavailable,
        Unavailable,
        LayeredSurface<'static, CheckedPass>,
        Unavailable,
        Unavailable,
        Unavailable,
    >;
    let mut surface = LayeredSurface::default();
    surface.layers.push(CheckedPass { depth: f32::NAN });
    let selected = CustomSurfaces::Layered(Box::new(surface));
    assert!(selected.validate().is_err());
}

#[test]
fn texture_bindings_traverse_every_selected_component() {
    use wc3::model::semantic::materials::bindings::TextureBinding;

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
fn programmed_input_validation_reaches_nested_components_at_publication() {
    use wc3::model::semantic::materials::programmed::{
        Material as ProgrammedMaterial, SamplingProgram, Standard, TextureLayer,
    };
    use wc3::model::semantic::profiles::M3;
    use wc3::model::semantic::{Material, Model};

    let mut layer = TextureLayer::default();
    layer.binding.sampling = SamplingProgram {
        noise_amplitude: f32::NAN,
        ..Default::default()
    };
    let mut standard = Standard::default();
    standard.diffuse.input = Some(layer);
    let mut builder = Model::<M3>::builder("invalid input");
    builder
        .add_material(Material {
            name: "surface".into(),
            surface: ProgrammedMaterial::Standard(Box::new(standard)),
        })
        .unwrap();
    assert!(builder.finish().is_err());
}

#[test]
fn model_rasterization_preserves_unknown_flags_and_edits_known_operations() {
    use wc3::model::semantic::materials::model_shader::{material_flags, rasterization};
    use wc3::model::semantic::materials::state::Operation;
    use wc3::model::semantic::values::model::MaterialFlags;

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
    use wc3::model::semantic::materials::bindings::TextureBinding;
    use wc3::model::semantic::profiles::Universal;
    use wc3::model::semantic::{Resources, Texture};

    let mut resources = Resources::<Universal>::builder();
    let source = resources.add_texture(Texture::file("generated-source.blp"));
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
        if let Link::Texture(id) = link {
            found |= id == source;
        }
    });
    assert!(found);
}
