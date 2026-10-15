use std::borrow::Cow;

use wc3::model::semantic::profiles::{m2, m3, mdlx, wmo};
use wc3::model::semantic::{
    Animated, AttachmentActivation, AttachmentIdentity, AttachmentPoint, ExternalAttachment,
    PlacementAppearance, Resource,
};

#[test]
fn attachment_points_share_activation_without_conflating_namespaces() {
    let point = AttachmentPoint::from(m2::Attachment {
        role: 7,
        enabled: Animated::constant(false),
    });
    assert!(matches!(point.identity, AttachmentIdentity::Role(7)));
    assert!(matches!(point.activation, AttachmentActivation::Enabled(ref value) if !value.default));
    let point = *m3::Attachment::try_from(point).unwrap_err();
    let recovered = m2::Attachment::try_from(point).unwrap();
    assert_eq!(recovered.role, 7);
    assert!(!recovered.enabled.default);

    let point = AttachmentPoint::from(m3::Attachment {
        name: Cow::Borrowed("weapon"),
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
        id: 42,
        visibility: Animated::constant(0.375),
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
    assert_eq!(recovered.id, 42);
    assert_eq!(recovered.visibility.default, 0.625);
}

#[test]
fn invalid_resource_narrowing_returns_the_complete_placement() {
    let source = mdlx::Attachment {
        resource: Cow::Owned("weapon.mdx".into()),
        id: 17,
        visibility: Animated::constant(0.5),
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
        tint: wmo::DoodadTint::Color([1, 2, 3, 4]),
        color_multiplier: Some(0.25),
        controls: Default::default(),
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
