use wc3::model::semantic::materials::contributions::{
    MaskedInput, NormalContributions, PairedInputs, SurfaceContribution,
};
use wc3::model::semantic::materials::programmed::{Property, Standard, TextureLayer};
use wc3::model::semantic::materials::PbrSurface;
use wc3::model::semantic::profiles::SurfacePayload;
use wc3::model::semantic::{Link, ValidateLinks};

#[test]
fn contribution_compositions_validate_inputs_masks_and_response_independently() {
    let value = SurfaceContribution {
        input: PairedInputs {
            primary: Some(1_f32),
            secondary: Some(f32::NAN),
        },
        parameters: f32::INFINITY,
    };
    let mut invalid = 0;
    value.visit_links(&mut |link| {
        if matches!(link, Link::InvalidValue) {
            invalid += 1;
        }
    });
    assert_eq!(invalid, 2);

    let normals = NormalContributions {
        base: None::<f32>,
        primary: MaskedInput {
            input: None,
            mask: Some(f32::NAN),
        },
        secondary: MaskedInput::default(),
        weights: vec![1_f32, 0.5, 0.25, f32::INFINITY],
    };
    let mut invalid = 0;
    normals.visit_links(&mut |link| {
        if matches!(link, Link::InvalidValue) {
            invalid += 1;
        }
    });
    assert_eq!(invalid, 2);
    assert_eq!(normals.weights.len(), 4);
}

#[test]
fn programmed_standard_keeps_sparse_slots_and_independent_weight_count() {
    let mut value = Standard::default();
    value.emissive.input.secondary.input = Some(TextureLayer::default());
    value.normals.primary.mask = Some(TextureLayer::default());
    value.normals.weights = vec![Property::default(); 5];
    assert!(value.emissive.input.primary.input.is_none());
    assert!(value.emissive.input.secondary.input.is_some());
    assert!(value.normals.primary.input.is_none());
    assert!(value.normals.primary.mask.is_some());
    assert_eq!(value.normals.weights.len(), 5);
}

#[test]
fn portable_constant_contributions_remain_validated() {
    let mut surface = PbrSurface::default();
    surface.base_color.parameters[2] = f32::NAN;
    assert!(surface.validate().is_err());
    surface.base_color.parameters = [1.; 4];
    surface.emissive.parameters[1] = f32::INFINITY;
    assert!(surface.validate().is_err());
}
