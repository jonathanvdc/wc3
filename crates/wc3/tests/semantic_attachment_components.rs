use std::borrow::Cow;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{FromFeature as SlotFromFeature, TryFromFeature as SlotTryFromFeature};

use wc3::model::profiles::{m2, m3, mdlx, wmo};
use wc3::model::{
    ChildModel, PlacementAppearance, ResourceLocator, Socket, SocketDefinition, SocketIdentity,
    WorldPlacementAppearance,
};

#[test]
fn attachment_points_share_activation_without_conflating_namespaces() {
    let mut slot_values = SlotValues::default();
    let point = Socket::from_feature(
        m2::Attachment {
            identity: 7,
            visibility: slot_values.add(false),
            control: (),
        },
        &mut slot_values,
    );
    assert!(matches!(point.identity, SocketIdentity::Role(7)));
    assert_eq!(slot_values[point.visibility], 0.);
    let point = *m3::Attachment::try_from_feature(point, &slot_values).unwrap_err();
    let recovered = m2::Attachment::try_from_feature(point, &slot_values).unwrap();
    assert_eq!(recovered.identity, 7);
    assert!(!slot_values[recovered.visibility]);

    let point = Socket::from_feature(
        m3::Attachment {
            identity: Cow::Borrowed("weapon"),
            visibility: (),
            control: 0x87654321,
        },
        &mut slot_values,
    );
    let point = *m2::Attachment::try_from_feature(point, &slot_values).unwrap_err();
    assert!(matches!(
        point.identity,
        SocketIdentity::Named(Cow::Borrowed("weapon"))
    ));
    let recovered = m3::Attachment::try_from_feature(point, &slot_values).unwrap();
    assert_eq!(recovered.control, 0x87654321);
}

#[test]
fn external_attachment_keeps_resource_identity_and_visibility() {
    let mut slot_values = SlotValues::default();
    let mut draft = wc3::model::Model::<wc3::model::profiles::Mdlx>::builder("resources");
    let resource =
        draft.add_model_resource(wc3::model::PathResource::Path(Cow::Borrowed(b"weapon.mdx")));
    let source = mdlx::Attachment {
        resource,
        socket: SocketDefinition {
            identity: 42,
            visibility: slot_values.add(0.375),
            control: (),
        },
        appearance: (),
    };
    let mut placement = ChildModel::from(source);
    assert_eq!(placement.resource, resource);
    let point = placement.socket.as_mut().unwrap();
    let value = &mut point.visibility;
    slot_values.set(*value, 0.625).unwrap();
    let recovered = mdlx::Attachment::try_from(placement).unwrap();
    assert_eq!(recovered.resource, resource);
    assert_eq!(recovered.socket.identity, 42);
    assert_eq!(slot_values[recovered.socket.visibility], 0.625);
}

#[test]
fn attachment_conversion_preserves_handles_without_inspecting_resource_addresses() {
    let mut slot_values = SlotValues::default();
    let mut draft = wc3::model::Model::<wc3::model::profiles::Universal>::builder(
        ModelMetadata::Mdlx(Default::default()),
    );
    let resource = draft.add_model_resource(ResourceLocator::Path(Cow::Borrowed(&[0xff])));
    let placement = ChildModel::from(mdlx::Attachment {
        resource,
        socket: SocketDefinition {
            identity: 17,
            visibility: slot_values.add(0.5),
            control: (),
        },
        appearance: (),
    });
    let recovered = mdlx::Attachment::try_from(placement).unwrap();
    assert_eq!(recovered.resource, resource);
}

#[test]
fn world_appearance_survives_rejected_attachment_export() {
    let mut slot_values = SlotValues::default();
    let mut draft = wc3::model::Model::<wc3::model::profiles::Universal>::builder(
        ModelMetadata::Mdlx(Default::default()),
    );
    let resource = draft.add_model_resource(ResourceLocator::Path(Cow::Borrowed(b"tree.m2")));
    let source = wmo::Doodad {
        resource,
        socket: (),
        appearance: WorldPlacementAppearance {
            tint: wmo::DoodadTint::Color([1, 2, 3, 4]),
            color_multiplier: Some(0.25),
            controls: SlotConstruct::construct(&mut slot_values),
        },
    };
    let placement = ChildModel::from(source.clone());
    assert!(placement.socket.is_none());
    let placement = *mdlx::Attachment::try_from(placement).unwrap_err();
    assert!(matches!(
        placement.appearance,
        PlacementAppearance::World {
            color_multiplier: Some(0.25),
            ..
        }
    ));
    assert_eq!(wmo::Doodad::try_from(placement).unwrap(), source);
}

#[test]
fn custom_attachment_parts_traverse_their_graph_relationships() {
    use wc3::model::profiles::Universal;
    use wc3::model::{ChildModelDefinition, Link, Model, NodeId, ValidateLinks};

    #[derive(Clone, Debug)]
    struct LinkedControl(NodeId);
    impl ValidateLinks for LinkedControl {
        fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
            self.0.visit_links(visitor);
            visitor(Link::InvalidValue);
        }
    }

    let mut foreign = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = foreign.add_named_node("foreign attachment target").unwrap();
    let attachment = ChildModelDefinition {
        resource: LinkedControl(node),
        socket: SocketDefinition {
            identity: LinkedControl(node),
            visibility: (),
            control: LinkedControl(node),
        },
        appearance: (),
    };
    let mut references = Vec::new();
    let mut invalid_values = 0;
    attachment.visit_links(&mut |link| match link {
        Link::Node(id) => references.push(id),
        Link::InvalidValue => invalid_values += 1,
        _ => panic!("unexpected relationship"),
    });
    assert_eq!(references, [node; 3]);
    assert_eq!(invalid_values, 3);
}
