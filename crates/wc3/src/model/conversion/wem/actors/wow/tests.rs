use super::*;
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{Model, Node};
fn reference<T: Target>(count: u32, index: u32) -> Ref<T> {
    Ref::from_section(count, index)
}

#[test]
fn old_particle_parameters_borrow_age_curves_without_fabricating_new_controls() {
    let mut document = RawDocument::new();
    let times = document.append(&[records::Real { value: 0.25 }]).unwrap();
    let colors = document
        .append(&[records::Vector3 {
            value: [0.2, 0.3, 0.4],
        }])
        .unwrap();
    let cells = document
        .append(&[records::U32 { value: 0x8000_0001 }])
        .unwrap();
    let source = records::M2ParticleEmitterPayload::<records::V0> {
        texture: u32::MAX,
        texture2: u32::MAX,
        texture3: u32::MAX,
        particle_id: -9,
        priority_plane: -13,
        speed: 31.,
        color_times: reference(1, times.index()),
        colors: reference(1, colors.index()),
        head_cell_times: reference(1, times.index()),
        head_cells: reference(1, cells.index()),
        ..Default::default()
    };
    let mut decoded = read(&document, &source, &[]).unwrap();
    assert!(decoded.colors.is_borrowed());
    assert_eq!(decoded.head_cells.get(0), Some(0x8000_0001));
    assert_eq!(decoded.texture2_scale, None);
    assert!(decoded.particle_model.is_none());
    let mut builder = Model::<Wem>::builder("particle");
    let node = builder.add_node(Node::named("emitter")).unwrap();
    let owner = builder.reserve_feature(node);
    let Bound::Scalar(property) = bind_property(
        &mut decoded,
        owner,
        records::M2ParticleProperty::Speed.raw(),
    )
    .unwrap() else {
        panic!("scalar control changed domain")
    };
    assert_eq!(decoded.speed.default, 31.);
    assert_eq!(
        property_code(&decoded, property.target()),
        Some(records::M2ParticleProperty::Speed.raw())
    );
    assert!(bind_property(&mut decoded, owner, u32::MAX).is_none());
    decoded.speed.default = 41.;
    let encoded =
        write::<records::V0>(&mut RawDocument::new(), &decoded, &BTreeMap::new()).unwrap();
    assert_eq!(encoded.speed, 41.);
    assert_eq!(encoded.priority_plane, -13);
    assert_eq!(encoded.particle_id, -9);
    assert!(write::<records::V19>(&mut RawDocument::new(), &decoded, &BTreeMap::new()).is_err());
}

#[test]
fn latest_particle_declarations_keep_independent_empty_asset_and_curve_presence() {
    let source = records::M2ParticleEmitterPayload::<records::V19> {
        texture: u32::MAX,
        texture2: u32::MAX,
        texture3: u32::MAX,
        particle_model: records::AssetKey {
            group: 19,
            id: u32::MAX,
            ..Default::default()
        },
        ..Default::default()
    };
    let document = RawDocument::new();
    let decoded = read(&document, &source, &[]).unwrap();
    assert!(decoded.alpha_cutoff_times.as_ref().unwrap().is_empty());
    assert_eq!(decoded.particle_model.as_ref().unwrap().numeric, None);
    assert_eq!(decoded.particle_model.as_ref().unwrap().namespace, 19);
    let encoded =
        write::<records::V19>(&mut RawDocument::new(), &decoded, &BTreeMap::new()).unwrap();
    assert_eq!(encoded.particle_model.group, 19);
    assert_eq!(encoded.particle_model.id, u32::MAX);
    assert_eq!(encoded.alpha_cutoff_times.entries, 0);
}
