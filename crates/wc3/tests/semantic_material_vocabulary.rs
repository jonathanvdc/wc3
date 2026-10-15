use wc3::model::semantic::controls::playback::ControllerProgram;
use wc3::model::semantic::materials::layered::{LayeredSurface, SurfaceLayer};
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
        source: Default::default(),
        texture: None,
        coordinate_set: 0,
        weight: Animated::constant(1.),
        transform: (),
        frame: Animated::constant(0),
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
        transform: TextureTransform::default(),
        source: input.source,
        texture: input.texture,
        coordinate_set: input.coordinate_set,
        weight: input.weight,
        frame: input.frame,
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
