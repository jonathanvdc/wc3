use std::borrow::Cow;

use wc3::model::profiles::{m2, m3, mdlx, wmo};
use wc3::model::{
    Animated, ChildModel, PlacementAppearance, Resource, Socket, SocketDefinition, SocketIdentity,
    WorldPlacementAppearance,
};

#[test]
fn attachment_points_share_activation_without_conflating_namespaces() {
    let point = Socket::from(m2::Attachment {
        identity: 7,
        visibility: Animated::constant(false),
        control: (),
    });
    assert!(matches!(point.identity, SocketIdentity::Role(7)));
    assert_eq!(point.visibility.default, 0.);
    let point = *m3::Attachment::try_from(point).unwrap_err();
    let recovered = m2::Attachment::try_from(point).unwrap();
    assert_eq!(recovered.identity, 7);
    assert!(!recovered.visibility.default);

    let point = Socket::from(m3::Attachment {
        identity: Cow::Borrowed("weapon"),
        visibility: (),
        control: 0x87654321,
    });
    let point = *m2::Attachment::try_from(point).unwrap_err();
    assert!(matches!(
        point.identity,
        SocketIdentity::Named(Cow::Borrowed("weapon"))
    ));
    let recovered = m3::Attachment::try_from(point).unwrap();
    assert_eq!(recovered.control, 0x87654321);
}

#[test]
fn external_attachment_keeps_resource_borrowing_and_visibility() {
    let source = mdlx::Attachment {
        resource: Cow::Borrowed("weapon.mdx"),
        socket: SocketDefinition {
            identity: 42,
            visibility: Animated::constant(0.375),
            control: (),
        },
        appearance: (),
    };
    let mut placement = ChildModel::from(source);
    assert!(matches!(
        placement.resource,
        Resource::Path(Cow::Borrowed(b"weapon.mdx"))
    ));
    let point = placement.socket.as_mut().unwrap();
    let value = &mut point.visibility;
    value.default = 0.625;
    let recovered = mdlx::Attachment::try_from(placement).unwrap();
    assert!(matches!(recovered.resource, Cow::Borrowed("weapon.mdx")));
    assert_eq!(recovered.socket.identity, 42);
    assert_eq!(recovered.socket.visibility.default, 0.625);
}

#[test]
fn invalid_resource_narrowing_returns_the_complete_placement() {
    let source = mdlx::Attachment {
        resource: Cow::Owned("weapon.mdx".into()),
        socket: SocketDefinition {
            identity: 17,
            visibility: Animated::constant(0.5),
            control: (),
        },
        appearance: (),
    };
    let mut placement = ChildModel::from(source);
    placement.resource = Resource::Path(Cow::Borrowed(&[0xff]));
    let placement = *mdlx::Attachment::try_from(placement).unwrap_err();
    assert!(matches!(
        placement.resource,
        Resource::Path(Cow::Borrowed([0xff]))
    ));
    assert!(matches!(
        placement.socket.unwrap().identity,
        SocketIdentity::Index(17)
    ));
}

#[test]
fn world_appearance_survives_rejected_attachment_export() {
    let source = wmo::Doodad {
        resource: Resource::Path(Cow::Borrowed(b"tree.m2")),
        socket: (),
        appearance: WorldPlacementAppearance {
            tint: wmo::DoodadTint::Color([1, 2, 3, 4]),
            color_multiplier: Some(0.25),
            controls: Default::default(),
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
    use wc3::model::{ChildModelDefinition, Link, Model, Node, NodeId, ValidateLinks};

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
