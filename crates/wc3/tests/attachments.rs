use std::borrow::Cow;
use wc3::model::profiles::{mdlx, Universal};
use wc3::model::ModelMetadata;
use wc3::model::PropertyValues as SlotValues;
use wc3::model::{
    AnimationProgram, AnimationSample, Attachment, ChildModel, Model, MountTarget, NodeFeature,
    PlacementAppearance, ResourceLocator, Socket, SocketDefinition, SocketIdentity,
};

#[test]
fn native_child_mounts_retain_fractional_visibility_without_inventing_points() {
    let mut slot_values = SlotValues::default();
    let mut model = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let resource = model.add_model_resource(ResourceLocator::Path(Cow::Borrowed(b"weapon.mdx")));
    let native = mdlx::Attachment {
        resource,
        socket: SocketDefinition {
            identity: 3,
            visibility: slot_values.add(0.25),
            control: (),
        },
        appearance: (),
    };
    let attachment = Attachment::Child(native.into()).into_owned();
    let node = model.add_named_node("hand").unwrap();
    let feature = model
        .add_feature(node, NodeFeature::Attachment(attachment.clone()))
        .unwrap();
    assert_eq!(
        attachment.mount(feature, node).unwrap().target,
        MountTarget::Socket(feature)
    );
    let point = Attachment::Socket(Socket {
        identity: SocketIdentity::Role(3),
        visibility: slot_values.add(0.),
        control: None,
    });
    model.import_values(&slot_values).unwrap();
    let model = model.finish().unwrap();
    let animation = AnimationProgram::compile(&model);
    let sample = animation.sample(AnimationSample::default()).unwrap();
    assert_eq!(attachment.visibility(&sample), 0.25);
    let restored = mdlx::Attachment::try_from(attachment.child().unwrap().clone()).unwrap();
    assert_eq!(restored.resource, resource);
    assert_eq!(restored.socket.identity, 3);
    assert_eq!(slot_values[restored.socket.visibility], 0.25);

    let placement = Attachment::Child(ChildModel {
        resource,
        socket: None,
        appearance: PlacementAppearance::Unmodified,
    });
    assert_eq!(
        placement.mount(feature, node).unwrap().target,
        MountTarget::Node(node)
    );
    assert_eq!(placement.visibility(&sample), 1.);
    assert!(point.mount(feature, node).is_none());
    assert_eq!(point.visibility(&sample), 0.);
}
