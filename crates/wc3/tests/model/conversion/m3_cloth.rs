use super::m3::{approximate, asset};
use wc3::model::{
    conversion,
    m3::{self, records as nr, Ref},
    semantic::profiles::m3 as surface,
};
#[test]
fn public_adapter_rebuilds_edited_cloth_density_and_node_bindings() {
    let original = asset();
    let native = conversion::m3::plan::<m3::StarCraft2>(&original, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root_id = native.model.root_id();
    let mut root = native.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = native.model.into_document();
    let enabled = document.append(&[nr::Byte { value: 1 }; 3]).unwrap();
    let palette = document.append(&[nr::U16 { value: 0 }]).unwrap();
    let bones = document.append(&[nr::U32 { value: 0 }; 3]).unwrap();
    let weights = document.append(&[nr::U32 { value: 255 }; 3]).unwrap();
    let colliders = document
        .append(&[nr::ClothCollider {
            bone: 0,
            radius: 2.,
            height: 3.,
            ..Default::default()
        }])
        .unwrap();
    let cloth = document
        .append(&[nr::ClothPhysics::<nr::V4> {
            density: 3.,
            anchor_control: 0x8000_0011,
            sim_enabled: Ref::from_section(3, enabled.index()),
            skin_bones: Ref::from_section(1, palette.index()),
            vertex_bones: Ref::from_section(3, bones.index()),
            vertex_weights: Ref::from_section(3, weights.index()),
            colliders: Ref::from_section(1, colliders.index()),
            active: nr::AnimationValue {
                initial_value: 1,
                null_value: 1,
                ..Default::default()
            },
            ..Default::default()
        }])
        .unwrap();
    root.cloth_physics = Ref::from_section(1, cloth.index());
    document.replace(root_id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let mut imported = conversion::m3::import(&source, &Default::default())
        .unwrap()
        .value;
    let (id, system) = imported
        .systems()
        .find(|(_, system)| matches!(system, surface::System::Cloth(_)))
        .unwrap();
    let mut system = system.clone();
    let surface::System::Cloth(cloths) = &mut system else {
        unreachable!()
    };
    assert!(cloths.cloths[0].anchors.is_borrowed());
    let anchor = cloths.cloths[0].anchors.get(0).unwrap()[0].unwrap();
    assert_eq!(cloths.cloths[0].colliders[0].bone, Some(anchor.node));
    cloths.cloths[0].density = 7.;
    cloths.cloths[0].colliders[0].radius = 4.;
    imported
        .edit(|builder| builder.set_system(id, system))
        .unwrap();
    let result = conversion::m3::plan::<m3::StarCraft2>(&imported, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = result.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let cloth = result
        .model
        .document()
        .resolve::<nr::ClothPhysics<nr::V4>>(root.cloth_physics.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(cloth.density, 7.);
    assert_eq!(cloth.anchor_control, 0x8000_0011);
    let collider = result
        .model
        .document()
        .resolve::<nr::ClothCollider>(cloth.colliders.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(collider.radius, 4.);
    assert_eq!(collider.bone, 0);
    let weights = result
        .model
        .document()
        .resolve::<nr::U32>(cloth.vertex_weights.cast())
        .unwrap();
    assert_eq!(weights.get(0).unwrap().unwrap().value, 255);
}
