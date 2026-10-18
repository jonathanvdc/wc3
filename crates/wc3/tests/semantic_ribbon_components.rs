use wc3::model::profiles::{m2, mdlx};
use wc3::model::{AngleTrailControls, SurfaceTrailRendering};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{GravityTrailControls, TrailAppearance};
use wc3::model::{TrailAtlasSlot, TrailControls, TrailEmitter, TrailRendering};

#[test]
fn trail_editing_preserves_wide_integer_rates_and_rejects_fractional_export() {
    let mut slot_values = SlotValues::default();
    let mut trail = TrailEmitter::from(mdlx::Ribbon {
        emission_rate: u32::MAX,
        appearance: TrailAppearance {
            texture_slot: slot_values.add(70_000),
            ..SlotConstruct::construct(&mut slot_values)
        },
        controls: GravityTrailControls {
            gravity: 9.8,
            ..SlotConstruct::construct(&mut slot_values)
        },
        ..SlotConstruct::construct(&mut slot_values)
    });
    assert_eq!(trail.emission_rate, f64::from(u32::MAX));
    slot_values.set(trail.geometry.height_above, 2.5).unwrap();
    let native = mdlx::Ribbon::try_from(trail.clone()).unwrap();
    assert_eq!(slot_values[native.geometry.height_above], 2.5);
    assert_eq!(native.emission_rate, u32::MAX);
    trail.emission_rate = 0.5;
    let rejected = mdlx::Ribbon::try_from(trail).unwrap_err();
    assert_eq!(rejected.emission_rate, 0.5);
}

#[test]
fn edge_trail_export_checks_numeric_width_without_conflating_direction_and_gravity() {
    let mut slot_values = SlotValues::default();
    let mut trail = TrailEmitter::from(mdlx::Ribbon::construct(&mut slot_values));
    trail.emission_rate = 0.5;
    trail.appearance.texture_slot = TrailAtlasSlot::Compact(slot_values.add(7));
    trail.rendering = TrailRendering::Surfaces(SurfaceTrailRendering {
        textures: vec![],
        materials: vec![],
    });
    trail.controls = TrailControls::EmissionAngle(AngleTrailControls {
        angle: 1.25,
        id: 9,
        enabled: slot_values.add(true),
        priority_plane: -2,
        color_index: 3,
        texture_transform: None,
    });
    let native = m2::Ribbon::try_from(trail.clone()).unwrap();
    assert_eq!(native.emission_rate, 0.5);
    assert_eq!(native.controls.angle, 1.25);
    trail.appearance.atlas[0] = 70_000;
    assert_eq!(
        m2::Ribbon::try_from(trail).unwrap_err().appearance.atlas[0],
        70_000
    );
}
