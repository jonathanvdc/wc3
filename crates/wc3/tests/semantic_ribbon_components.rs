use wc3::model::profiles::{m2, mdlx};
use wc3::model::{
    Construct, M2RibbonControls, M2RibbonRendering, MdlxRibbonControls, PropertyValues,
    RibbonEmitter, TrailAppearance,
};

#[test]
fn mdlx_ribbon_preserves_wide_integer_rate_and_property_identity() {
    let mut values = PropertyValues::default();
    let source = mdlx::RibbonEmitter {
        emission_rate: u32::MAX,
        appearance: TrailAppearance {
            texture_slot: values.add(70_000),
            ..Construct::construct(&mut values)
        },
        controls: MdlxRibbonControls {
            gravity: 9.8,
            ..Construct::construct(&mut values)
        },
        ..Construct::construct(&mut values)
    };
    let mut emitter = RibbonEmitter::from(source.clone());
    let RibbonEmitter::Mdlx(trail) = &mut emitter else {
        panic!()
    };
    values.set(trail.geometry.height_above, 2.5).unwrap();
    let restored = mdlx::RibbonEmitter::try_from(emitter).unwrap();
    assert_eq!(restored, source);
    assert_eq!(values[restored.geometry.height_above], 2.5);
    assert_eq!(restored.emission_rate, u32::MAX);
}

#[test]
fn m2_ribbon_preserves_fractional_rate_angle_and_typed_atlas_slot() {
    let mut values = PropertyValues::default();
    let source = m2::RibbonEmitter {
        emission_rate: 0.5,
        controls: M2RibbonControls {
            angle: 1.25,
            id: 9,
            enabled: values.add(true),
            priority_plane: -2,
            color_index: 3,
            texture_transform: None,
        },
        rendering: M2RibbonRendering {
            textures: vec![],
            materials: vec![],
        },
        appearance: TrailAppearance {
            texture_slot: values.add(7_u16),
            ..Construct::construct(&mut values)
        },
        lifetime: 1.,
        geometry: Construct::construct(&mut values),
    };
    let before = format!("{source:?}");
    let emitter = RibbonEmitter::from(source);
    let rejected = mdlx::RibbonEmitter::try_from(emitter).unwrap_err();
    let restored = m2::RibbonEmitter::try_from(*rejected).unwrap();
    assert_eq!(format!("{restored:?}"), before);
    assert_eq!(restored.emission_rate, 0.5);
    assert_eq!(restored.controls.angle, 1.25);
    assert_eq!(values[restored.appearance.texture_slot], 7);
}
