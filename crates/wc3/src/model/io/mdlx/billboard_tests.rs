use super::*;
use crate::model::formats::mdlx::scene::{Node as NativeNode, NodeFlags};
use crate::model::formats::mdlx::{Document, V800};
use crate::model::properties::BillboardFlags;
use crate::model::{
    Billboard, BillboardAxis, BillboardFrame, BillboardScale, EulerLocks, TransformSemantics,
};

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
        assert_eq!(semantic.transform_semantics, TransformSemantics::PivotLocal);
        let original = semantic.properties.billboard_flags;
        assert_eq!(
            original,
            BillboardFlags {
                enabled: flags.billboarded(),
                x: flags.billboard_lock_x(),
                y: flags.billboard_lock_y(),
                z: flags.billboard_lock_z(),
            }
        );
        let expected = if flags.billboarded() {
            Billboard::Spherical {
                frame: BillboardFrame::Yz,
                scale: BillboardScale::SignedFactors,
            }
        } else if flags.billboard_lock_x() {
            Billboard::Axial(BillboardAxis::X)
        } else if flags.billboard_lock_y() {
            Billboard::Axial(BillboardAxis::Y)
        } else if flags.billboard_lock_z() {
            Billboard::Axial(BillboardAxis::Z)
        } else {
            Billboard::None
        };
        assert_eq!(semantic.billboard, expected);
        let output = export::<V800>(&model, &ExportOptions::default())
            .unwrap()
            .value;
        assert_eq!(output.nodes()[0].flags.bits() & 0x78, flags.bits());
    }
}

#[test]
fn semantic_edits_do_not_reactivate_overridden_axis_declarations() {
    for mode in [Billboard::None, Billboard::Axial(BillboardAxis::Y)] {
        let source = Document::<V800>::decode_mdl("Version { FormatVersion 800, } Model \"flags\" {} Helper \"helper\" { ObjectId 0, Billboarded, BillboardedLockX, BillboardedLockZ, } PivotPoints 1 { { 0, 0, 0 }, }").unwrap();
        let mut model = import(&source, &ImportOptions::default()).unwrap().value;
        let (id, node) = model.nodes().iter().next().unwrap();
        let mut node = node.clone();
        node.billboard = mode;
        model.edit(|builder| builder.set_node(id, node)).unwrap();
        let output = export::<V800>(&model, &ExportOptions::default())
            .unwrap()
            .value;
        assert_eq!(
            output.nodes()[0].flags.bits() & 0x78,
            if mode == Billboard::None { 0 } else { 0x20 }
        );
    }
}

#[test]
fn incompatible_effective_operations_require_explicit_export_loss() {
    let source = Document::<V800>::decode_mdl("Version { FormatVersion 800, } Model \"flags\" {} Helper \"helper\" { ObjectId 0, } PivotPoints 1 { { 0, 0, 0 }, }").unwrap();
    for mode in [
        Billboard::Euler(EulerLocks::X),
        Billboard::Spherical {
            frame: BillboardFrame::Camera,
            scale: BillboardScale::SignedFactors,
        },
        Billboard::Spherical {
            frame: BillboardFrame::Yz,
            scale: BillboardScale::BasisLengths,
        },
    ] {
        let mut model = import(&source, &ImportOptions::default()).unwrap().value;
        let (id, node) = model.nodes().iter().next().unwrap();
        let mut node = node.clone();
        node.billboard = mode;
        model.edit(|builder| builder.set_node(id, node)).unwrap();
        assert!(export::<V800>(&model, &ExportOptions::default()).is_err());
    }
}

#[test]
fn transform_semantics_are_checked_independently_of_billboards() {
    let source = Document::<V800>::decode_mdl("Version { FormatVersion 800, } Model \"flags\" {} Helper \"helper\" { ObjectId 0, DontInheritRotation, } PivotPoints 1 { { 0, 0, 0 }, }").unwrap();
    let mut model = import(&source, &ImportOptions::default()).unwrap().value;
    let (id, node) = model.nodes().iter().next().unwrap();
    let mut node = node.clone();
    node.transform_semantics = TransformSemantics::Affine;
    model.edit(|builder| builder.set_node(id, node)).unwrap();
    assert!(export::<V800>(&model, &ExportOptions::default()).is_err());
}
