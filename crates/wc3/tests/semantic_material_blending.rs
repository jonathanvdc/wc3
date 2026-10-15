use wc3::model::materials::state::BlendOperations;
use wc3::model::materials::{layered, world};
use wc3::model::values::{model, programmed};

#[test]
fn model_and_programmed_blending_preserve_every_known_and_unknown_selector() {
    for raw in (0..=8).chain([u16::MAX]) {
        assert_eq!(model::BlendMode::from_raw(raw).raw(), raw);
    }
    for raw in (0..=7).chain([u32::MAX]) {
        assert_eq!(programmed::BlendMode::from_raw(raw).raw(), raw);
    }
    assert_eq!(model::BlendMode::from_raw(3), BlendOperations::Additive);
    assert_eq!(
        model::BlendMode::from_raw(4),
        BlendOperations::AdditiveAlpha(())
    );
    assert_eq!(
        programmed::BlendMode::from_raw(2),
        BlendOperations::Additive
    );
    assert_eq!(
        programmed::BlendMode::from_raw(3),
        BlendOperations::AdditiveAlpha(())
    );
    assert_eq!(
        model::BlendMode::from_raw(7),
        BlendOperations::Extension(model::ModelBlend::BlendAdd)
    );
}

#[test]
fn selected_material_blends_use_shared_operations_and_explicit_extra_domains() {
    let cutout: layered::Blend = BlendOperations::AlphaKey(());
    assert_eq!(cutout, layered::Blend::AlphaKey(()));
    let world: world::Blend = BlendOperations::Extension(world::WorldBlend::ModulateAdd);
    assert_eq!(
        world,
        world::Blend::Extension(world::WorldBlend::ModulateAdd)
    );
    assert_eq!(layered::Blend::default(), BlendOperations::Opaque);
    assert_eq!(world::Blend::default(), BlendOperations::Opaque);
    assert_eq!(programmed::BlendMode::default(), BlendOperations::Opaque);
}
