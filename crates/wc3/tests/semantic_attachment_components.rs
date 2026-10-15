use std::borrow::Cow;

use wc3::model::profiles::{m2, m3, mdlx, wmo};
use wc3::model::{
    Animated, AttachmentActivation, AttachmentIdentity, AttachmentPoint, AttachmentPointDefinition,
    ExternalAttachment, PlacementAppearance, Resource, WorldPlacementAppearance,
};

#[test]
fn attachment_points_share_activation_without_conflating_namespaces() {
    let point = AttachmentPoint::from(m2::Attachment {
        identity: 7,
        activation: Animated::constant(false),
        control: (),
    });
    assert!(matches!(point.identity, AttachmentIdentity::Role(7)));
    assert!(matches!(point.activation, AttachmentActivation::Enabled(ref value) if !value.default));
    let point = *m3::Attachment::try_from(point).unwrap_err();
    let recovered = m2::Attachment::try_from(point).unwrap();
    assert_eq!(recovered.identity, 7);
    assert!(!recovered.activation.default);

    let point = AttachmentPoint::from(m3::Attachment {
        identity: Cow::Borrowed("weapon"),
        activation: (),
        control: 0x87654321,
    });
    let point = *m2::Attachment::try_from(point).unwrap_err();
    assert!(matches!(
        point.identity,
        AttachmentIdentity::Named(Cow::Borrowed("weapon"))
    ));
    let recovered = m3::Attachment::try_from(point).unwrap();
    assert_eq!(recovered.control, 0x87654321);
}

#[test]
fn external_attachment_keeps_resource_borrowing_and_visibility() {
    let source = mdlx::Attachment {
        resource: Cow::Borrowed("weapon.mdx"),
        point: AttachmentPointDefinition {
            identity: 42,
            activation: Animated::constant(0.375),
            control: (),
        },
        appearance: (),
    };
    let mut placement = ExternalAttachment::from(source);
    assert!(matches!(
        placement.resource,
        Resource::Path(Cow::Borrowed(b"weapon.mdx"))
    ));
    let point = placement.point.as_mut().unwrap();
    let AttachmentActivation::Visibility(value) = &mut point.activation else {
        panic!()
    };
    value.default = 0.625;
    let recovered = mdlx::Attachment::try_from(placement).unwrap();
    assert!(matches!(recovered.resource, Cow::Borrowed("weapon.mdx")));
    assert_eq!(recovered.point.identity, 42);
    assert_eq!(recovered.point.activation.default, 0.625);
}

#[test]
fn invalid_resource_narrowing_returns_the_complete_placement() {
    let source = mdlx::Attachment {
        resource: Cow::Owned("weapon.mdx".into()),
        point: AttachmentPointDefinition {
            identity: 17,
            activation: Animated::constant(0.5),
            control: (),
        },
        appearance: (),
    };
    let mut placement = ExternalAttachment::from(source);
    placement.resource = Resource::Path(Cow::Borrowed(&[0xff]));
    let placement = *mdlx::Attachment::try_from(placement).unwrap_err();
    assert!(matches!(
        placement.resource,
        Resource::Path(Cow::Borrowed([0xff]))
    ));
    assert!(matches!(
        placement.point.unwrap().identity,
        AttachmentIdentity::Index(17)
    ));
}

#[test]
fn world_appearance_survives_rejected_attachment_export() {
    let source = wmo::Doodad {
        resource: Resource::Path(Cow::Borrowed(b"tree.m2")),
        point: (),
        appearance: WorldPlacementAppearance {
            tint: wmo::DoodadTint::Color([1, 2, 3, 4]),
            color_multiplier: Some(0.25),
            controls: Default::default(),
        },
    };
    let placement = ExternalAttachment::from(source.clone());
    assert!(placement.point.is_none());
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
    use wc3::model::{ExternalAttachmentDefinition, Link, Model, Node, NodeId, ValidateLinks};

    #[derive(Clone, Debug)]
    struct LinkedControl(NodeId);
    impl ValidateLinks for LinkedControl {
        fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
            self.0.visit_links(visitor);
            visitor(Link::InvalidValue);
        }
    }

    let mut foreign = Model::<Universal>::builder("foreign owner");
    let node = foreign
        .add_node(Node::named("foreign attachment target"))
        .unwrap();
    let attachment = ExternalAttachmentDefinition {
        resource: LinkedControl(node),
        point: AttachmentPointDefinition {
            identity: LinkedControl(node),
            activation: (),
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
