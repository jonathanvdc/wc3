use std::borrow::Cow;
use wc3::model::profiles::{m3, mdlx, Universal};
use wc3::model::{
    Animated, AnimationProgram, AnimationSample, Attachment, ChildModel, Model, MountTarget, Node,
    NodeFeature, PlacementAppearance, Resource, Socket, SocketDefinition, SocketIdentity,
};

#[test]
fn owned_attachment_preserves_native_identity_controls_and_activation() {
    let native = m3::Attachment {
        identity: Cow::Borrowed("weapon"),
        visibility: (),
        control: 47,
    };
    let attachment = Attachment::Socket(native.into()).into_owned();
    let point = attachment.socket().unwrap();
    assert!(matches!(&point.identity, SocketIdentity::Named(Cow::Owned(name)) if name == "weapon"));
    assert_eq!(point.control, Some(47));
    assert_eq!(point.visibility.default, 1.);
    let restored = m3::Attachment::try_from(point.clone()).unwrap();
    assert_eq!(restored.control, 47);
    assert_eq!(restored.identity, "weapon");
}

#[test]
fn native_child_mounts_retain_fractional_visibility_without_inventing_points() {
    let native = mdlx::Attachment {
        resource: Cow::Borrowed("weapon.mdx"),
        socket: SocketDefinition {
            identity: 3,
            visibility: Animated::constant(0.25),
            control: (),
        },
        appearance: (),
    };
    let attachment = Attachment::Child(native.into()).into_owned();
    let mut model = Model::<Universal>::builder("child mounts");
    let node = model.add_node(Node::named("hand")).unwrap();
    let feature = model
        .add_feature(node, NodeFeature::Attachment(attachment.clone()))
        .unwrap();
    assert_eq!(
        attachment.mount(feature, node).unwrap().target,
        MountTarget::Socket(feature)
    );
    let model = model.finish().unwrap();
    let animation = AnimationProgram::compile(&model);
    let sample = animation.sample(AnimationSample::default()).unwrap();
    assert_eq!(attachment.visibility(&sample), 0.25);
    let restored = mdlx::Attachment::try_from(attachment.child().unwrap().clone()).unwrap();
    assert_eq!(restored.resource, "weapon.mdx");
    assert_eq!(restored.socket.identity, 3);
    assert_eq!(restored.socket.visibility.default, 0.25);

    let placement = Attachment::Child(ChildModel {
        resource: Resource::Path(Cow::Borrowed(b"doodad.m2")),
        socket: None,
        appearance: PlacementAppearance::Unmodified,
    });
    assert_eq!(
        placement.mount(feature, node).unwrap().target,
        MountTarget::Node(node)
    );
    assert_eq!(placement.visibility(&sample), 1.);
    let point = Attachment::Socket(Socket {
        identity: SocketIdentity::Role(3),
        visibility: Animated::constant(0.),
        control: None,
    });
    assert!(point.mount(feature, node).is_none());
    assert_eq!(point.visibility(&sample), 0.);
}
