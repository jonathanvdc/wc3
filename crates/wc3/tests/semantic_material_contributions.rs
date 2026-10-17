use wc3::model::materials::contributions::{
    MaskedInput, NormalContributions, PairedInputs, SurfaceContribution,
};
use wc3::model::materials::m3::{Property, StandardMaterialStep, TextureLayer};
use wc3::model::{Link, ValidateLinks};

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
    let mut value = StandardMaterialStep::default();
    value.emissive.input.secondary.input = Some(TextureLayer::default());
    value.normals.primary.mask = Some(TextureLayer::default());
    value.normals.weights = vec![Property::default(); 5];
    assert!(value.emissive.input.primary.input.is_none());
    assert!(value.emissive.input.secondary.input.is_some());
    assert!(value.normals.primary.input.is_none());
    assert!(value.normals.primary.mask.is_some());
    assert_eq!(value.normals.weights.len(), 5);
}
