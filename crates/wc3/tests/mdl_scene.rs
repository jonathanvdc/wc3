use wc3::model::geometry::CollisionShape;
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::scene::{EventObject, Light, NodeFlags};
use wc3::model::{mdl, mdx, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900};

fn roundtrip<T: mdl::Read + mdl::Write + mdx::Read + mdx::Write>(value: &T) {
    let binary = value.encode_mdx().unwrap();
    let text = value.encode_mdl().unwrap();
    let decoded = T::decode_mdl(&text).unwrap_or_else(|error| panic!("{error}: {text}"));
    assert_eq!(decoded.encode_mdx().unwrap(), binary, "{text}");
}
#[test]
fn light_roundtrips_at_all_versions() {
    macro_rules! check { ($($version:ty),*) => { $( {
        let light = Light::<$version>::decode_mdl("Light \"a\" { ObjectId 0, Omnidirectional, Translation 0 { Linear, } Visibility 0 { DontInterp, } Intensity 0 { Hermite, } static Color { 0.1, 0.2, 0.3 }, }").unwrap();
        assert_eq!(light.node.flags.bits(), 0x200);
        assert_eq!(light.color, [0.1, 0.2, 0.3]);
        roundtrip(&light);
    } )* }; }
    check!(V800, V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800);
    for (flag, id) in [("Directional", 1), ("Ambient", 2)] {
        let light =
            Light::<V800>::decode_mdl(&format!("Light \"a\" {{ ObjectId 0, {flag}, }}")).unwrap();
        assert_eq!(light.light_type, id);
        roundtrip(&light);
    }
}
#[test]
fn light_versioned_fields_and_tracks() {
    roundtrip(
        &Light::<V1200>::decode_mdl(
            "Light \"a\" { ObjectId 0, Ambient, static ShadowIntensity -0.0, }",
        )
        .unwrap(),
    );
    roundtrip(&Light::<V1300>::decode_mdl("Light \"a\" { ObjectId 0, Ambient, ShadowCasting, ShadowCastingStart 1 { Linear, GlobalSeqId 3, -10: 8.0, } static ShadowCastingEnd 20.0, }").unwrap());
    let text = "Light \"a\" { ObjectId 0, Directional, static QuadraticFalloff 0.002, Damping 0 { Bezier, } static LinearFalloff -0.0, }";
    roundtrip(&Light::<V1600>::decode_mdl(text).unwrap());
    roundtrip(&Light::<V1800>::decode_mdl(text).unwrap());
    for field in [
        "static ShadowIntensity 0.0,",
        "ShadowCasting,",
        "static ShadowCastingStart 0.0,",
        "ShadowCastingEnd 0 { Linear, }",
        "static QuadraticFalloff 0.0005,",
        "Damping 0 { Linear, }",
    ] {
        let text = format!("Light \"a\" {{ ObjectId 0, Ambient, {field} }}");
        assert!(Light::<V800>::decode_mdl(&text).is_err(), "{field}");
    }
    assert!(
        Light::<V1200>::decode_mdl("Light \"a\" { ObjectId 0, Ambient, ShadowCasting, }").is_err()
    );
    assert!(Light::<V1400>::decode_mdl(
        "Light \"a\" { ObjectId 0, Ambient, static LinearFalloff 0.0, }"
    )
    .is_err());
}
#[test]
fn light_rejects_ambiguous_and_unrepresentable_data() {
    for body in [
        "",
        "Omnidirectional, Directional,",
        "Ambient, Ambient,",
        "Ambient, static Visibility 0.0,",
        "Ambient, Visibility 0 { Linear, } Visibility 0 { Linear, }",
        "Ambient, static Intensity 0.0, Intensity 0 { Linear, }",
        "Ambient, ShadowIntensity 0 { Linear, }",
    ] {
        let text = format!("Light \"a\" {{ ObjectId 0, {body} }}");
        assert!(Light::<V1800>::decode_mdl(&text).is_err(), "{body}");
    }
    let mut light =
        Light::<V800>::decode_mdl("Light \"a\" { ObjectId 0, Ambient, Intensity 0 { Linear, } }")
            .unwrap();
    light.intensity = 1.0;
    assert!(light.encode_mdl().is_err());
    light.intensity = 0.0;
    light.light_type = 99;
    assert!(light.encode_mdl().is_err());
    light.light_type = 0;
    light.node.flags = NodeFlags(0x400);
    assert!(light.encode_mdl().is_err());
    let light = Light::<V1300>::decode_mdl("Light \"a\" { ObjectId 0, Ambient, }").unwrap();
    let mut bytes = light.encode_mdx().unwrap();
    let node_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let casting_offset = 4 + node_size + 4;
    bytes[casting_offset..casting_offset + 4].copy_from_slice(&2u32.to_le_bytes());
    assert!(Light::<V1300>::decode_mdx(&bytes)
        .unwrap()
        .encode_mdl()
        .is_err());
}
#[test]
fn event_frames_and_sequence_roundtrip() {
    for body in [
        "0 {}",
        "3 { GlobalSeqId 7, -2147483648, 0, 2147483647, }",
        "2 { -10, -10, }",
    ] {
        let text = format!("EventObject \"e\" {{ ObjectId 0, EventTrack {body} }}");
        let event = EventObject::decode_mdl(&text).unwrap();
        assert_eq!(event.node.flags.bits(), 0x400);
        roundtrip(&event);
    }
    let event = EventObject::decode_mdl(
        "EventObject \"e\" { EventTrack 1 { 4, } ObjectId 0, Translation 0 { Linear, } }",
    )
    .unwrap();
    assert_eq!(event.global_sequence_id, u32::MAX);
    assert_eq!(event.frames.as_slice(), [4]);
    roundtrip(&event);
}
#[test]
fn event_rejects_counts_duplicates_and_track_syntax() {
    for body in [
        "",
        "EventTrack 0 {} EventTrack 0 {}",
        "EventTrack 1 {}",
        "EventTrack 0 { 1, }",
        "EventTrack 1 { 2147483648, }",
        "EventTrack 0 { Linear, }",
        "EventTrack 0 { GlobalSeqId 1, GlobalSeqId 2, }",
        "EventTrack 1 { 0: 1, }",
    ] {
        let text = format!("EventObject \"e\" {{ ObjectId 0, {body} }}");
        assert!(EventObject::decode_mdl(&text).is_err(), "{body}");
    }
    let mut event =
        EventObject::decode_mdl("EventObject \"e\" { ObjectId 0, EventTrack 0 {} }").unwrap();
    event.node.flags = NodeFlags(0);
    assert!(event.encode_mdl().is_err());
}
#[test]
fn all_collision_shapes_roundtrip() {
    for (kind, vertices, radius) in [
        ("Box", "2 { { 1, 2, 3 }, { 4, 5, 6 }, }", ""),
        ("Plane", "2 { { 1, 2, 3 }, { 4, 5, 6 }, }", ""),
        ("Sphere", "1 { { 1, 2, 3 }, }", "BoundsRadius 8.0,"),
        (
            "Cylinder",
            "2 { { 1, 2, 3 }, { 4, 5, 6 }, }",
            "BoundsRadius -0.0,",
        ),
    ] {
        let text =
            format!("CollisionShape \"c\" {{ {radius} Vertices {vertices} ObjectId 0, {kind}, }}");
        let shape = CollisionShape::decode_mdl(&text).unwrap();
        assert_eq!(shape.node.flags.bits(), 0x2000);
        roundtrip(&shape);
    }
}
#[test]
fn collision_rejects_inconsistent_geometry() {
    for body in [
        "Vertices 0 {}",
        "Box, Sphere, Vertices 1 { { 0, 0, 0 }, } BoundsRadius 1.0,",
        "Sphere, Vertices 2 { { 0, 0, 0 }, { 1, 1, 1 }, } BoundsRadius 1.0,",
        "Box, Vertices 1 { { 0, 0, 0 }, }",
        "Cylinder, Vertices 2 { { 0, 0, 0 }, { 1, 1, 1 }, }",
        "Box, Vertices 2 { { 0, 0, 0 }, { 1, 1, 1 }, } BoundsRadius 0.0,",
        "Sphere, Vertices 1 { { 0, 0, 0 }, } BoundsRadius 1.0, BoundsRadius 2.0,",
        "Sphere, Vertices 0 { { 0, 0, 0 }, } BoundsRadius 1.0,",
    ] {
        let text = format!("CollisionShape \"c\" {{ ObjectId 0, {body} }}");
        assert!(CollisionShape::decode_mdl(&text).is_err(), "{body}");
    }
}

#[test]
fn independent_binary_record_fixtures_roundtrip() {
    use wc3::model::animation::{AnimationTrack, LightDamping};
    use wc3::model::scene::{LightFalloff, LightShadowRange, LightTrack, Node};
    let mut node = Node::new("fixture", 7).unwrap();
    node.flags = NodeFlags(0x200);
    let mut light = Light::<V1800>::new(node.clone(), 1);
    light.attenuation_start = 3.0;
    light.attenuation_end = 40.0;
    light.color = [0.2, 0.4, 0.8];
    light.intensity = 2.0;
    light.ambient_intensity = 0.5;
    light.set_shadow_casting(true);
    light.set_shadow_intensity(0.75);
    light.set_shadow_casting_range(LightShadowRange {
        start: 4.0,
        end: 50.0,
    });
    light.set_falloff(LightFalloff {
        quadratic: 0.0005,
        linear: -0.0,
        damping: 0.001,
    });
    roundtrip(&light);
    let mut old_light = Light::<V800>::new(node.clone(), 0);
    old_light.tracks = (&[LightTrack::Damping(
        AnimationTrack::<LightDamping>::linear(vec![], None).unwrap(),
    )])
        .to_vec();
    assert!(old_light.encode_mdl().is_err());
    node.flags = NodeFlags(0x400);
    roundtrip(&EventObject::new(node.clone(), 2, &[50, -20, 50]));
    node.flags = NodeFlags(0x2000);
    let points = [[-0.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
    for shape in [
        CollisionShape::new_box(node.clone(), points),
        CollisionShape::new_plane(node.clone(), points),
        CollisionShape::new_sphere(node.clone(), points[0], 7.0),
        CollisionShape::new_cylinder(node, points, 8.0),
    ] {
        roundtrip(&shape);
    }
}
