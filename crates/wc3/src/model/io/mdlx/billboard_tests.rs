use super::*;
use crate::model::formats::mdlx::scene::{Node as NativeNode, NodeFlags};
use crate::model::formats::mdlx::{Document, V800};
use crate::model::Billboard;

#[test]
fn every_native_billboard_flag_combination_preserves_behavior_and_wire_bits() {
    for flags in (0..16).map(|bits| NodeFlags(bits << 3)) {
        let mut source =
            Document::<V800>::decode_mdl("Version { FormatVersion 800, } Model \"flags\" {}")
                .unwrap();
        let mut node = NativeNode::new("helper", 0).unwrap();
        node.flags = flags;
        source.set_helpers([node]);
        source.set_pivot_points(&[[0.; 3]]);
        let model = import(&source, &ImportOptions::default()).unwrap().value;
        let semantic = model.nodes().iter().next().unwrap().1;
        if flags.billboarded() {
            assert_eq!(semantic.billboard, Billboard::Spherical);
            let locks = semantic.controls.spherical_axis_locks;
            assert_eq!(
                (locks.x, locks.y, locks.z),
                (
                    flags.billboard_lock_x(),
                    flags.billboard_lock_y(),
                    flags.billboard_lock_z()
                )
            );
        } else if flags.billboard_lock_x() || flags.billboard_lock_y() || flags.billboard_lock_z() {
            assert_eq!(
                semantic.billboard,
                Billboard::Locked {
                    enabled: true,
                    x: flags.billboard_lock_x(),
                    y: flags.billboard_lock_y(),
                    z: flags.billboard_lock_z()
                }
            );
        } else {
            assert_eq!(semantic.billboard, Billboard::None);
        }
        let output = plan::<V800>(&model, &ExportOptions::default())
            .unwrap()
            .execute()
            .unwrap()
            .value;
        assert_eq!(output.nodes()[0].flags.bits() & 0x78, flags.bits());
    }
}

#[test]
fn changing_full_billboard_to_none_does_not_reactivate_masked_axis_locks() {
    let source=Document::<V800>::decode_mdl("Version { FormatVersion 800, } Model \"flags\" {} Helper \"helper\" { ObjectId 0, Billboarded, BillboardedLockX, } PivotPoints 1 { { 0, 0, 0 }, }").unwrap();
    let mut model = import(&source, &ImportOptions::default()).unwrap().value;
    let (id, node) = model.nodes().iter().next().unwrap();
    let mut node = node.clone();
    node.billboard = Billboard::None;
    model.edit(|builder| builder.set_node(id, node)).unwrap();
    let output = plan::<V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.nodes()[0].flags.bits() & 0x78, 0);
}

#[test]
fn disabled_semantic_axis_locks_cannot_silently_become_active_native_flags() {
    let source=Document::<V800>::decode_mdl("Version { FormatVersion 800, } Model \"flags\" {} Helper \"helper\" { ObjectId 0, } PivotPoints 1 { { 0, 0, 0 }, }").unwrap();
    let mut model = import(&source, &ImportOptions::default()).unwrap().value;
    let (id, node) = model.nodes().iter().next().unwrap();
    let mut node = node.clone();
    node.billboard = Billboard::Locked {
        enabled: false,
        x: true,
        y: false,
        z: false,
    };
    model.edit(|builder| builder.set_node(id, node)).unwrap();
    assert!(plan::<V800>(&model, &ExportOptions::default()).is_err());
}
