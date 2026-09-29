use super::*;

#[test]
fn texture_and_sequence_roundtrip_through_mdx() {
    let source = r#"Bitmap { WrapHeight, ReplaceableId 2, Image "Textures\Unit.blp", WrapWidth, }"#;
    let texture = Texture::decode_mdl(source).unwrap();
    assert_eq!(texture.path.text(), "Textures\\Unit.blp");
    assert_eq!(texture.flags.bits(), 3);
    assert_eq!(texture.replaceable_id, 2);
    let canonical = print(&texture);
    assert_eq!(canonical, "Bitmap {\n\tImage \"Textures\\Unit.blp\",\n\tReplaceableId 2,\n\tWrapWidth,\n\tWrapHeight,\n}\n");
    let bytes = texture.encode_mdx().unwrap();
    assert_eq!(
        Texture::decode_mdl(&print(&Texture::decode_mdx(&bytes).unwrap()))
            .unwrap()
            .encode_mdx()
            .unwrap(),
        bytes
    );
    assert_eq!(
        Texture::decode_mdl("Bitmap {}").unwrap(),
        Texture::new("").unwrap()
    );

    let source = r#"Anim "Walk" { SyncPoint 1, BoundsRadius 85.0, MaximumExtent { 55.0, 55.0, 105.0 },
        MinimumExtent { -55.0, -55.0, 0.0 }, Rarity -0.0, MoveSpeed 270.0, NonLooping, Interval { 3334, 6667 }, }"#;
    let sequence = Sequence::decode_mdl(source).unwrap();
    assert_eq!(sequence.interval, [3334, 6667]);
    assert_eq!(sequence.sync_point, 1);
    assert!(sequence.flags.non_looping());
    let bytes = sequence.encode_mdx().unwrap();
    assert_eq!(
        Sequence::decode_mdl(&print(&Sequence::decode_mdx(&bytes).unwrap()))
            .unwrap()
            .encode_mdx()
            .unwrap(),
        bytes
    );
    assert!(print(&sequence).contains("Rarity -0.0,"));
    assert_eq!(
        Sequence::decode_mdl("Anim \"Stand\" { Interval { 0, 1000 }, }").unwrap(),
        Sequence::new("Stand", [0, 1000]).unwrap()
    );
}

#[test]
fn record_readers_reject_duplicates_unknown_fields_missing_values_and_trailing_input() {
    assert_eq!(
        Texture::decode_mdl("Bitmap { Image \"a\", Image \"b\", }")
            .unwrap_err()
            .kind,
        ReadErrorKind::DuplicateField
    );
    assert_eq!(
        Texture::decode_mdl("Bitmap { WrapWidth, WrapWidth, }")
            .unwrap_err()
            .kind,
        ReadErrorKind::DuplicateField
    );
    assert_eq!(
        Sequence::decode_mdl("Anim \"A\" {}").unwrap_err().kind,
        ReadErrorKind::MissingField("Interval")
    );
    assert_eq!(
        Sequence::decode_mdl("Anim \"A\" { Interval { 0, 1 }, SyncPoint 1, SyncPoint 2, }")
            .unwrap_err()
            .kind,
        ReadErrorKind::DuplicateField
    );
    for source in [
        "Bitmap { Image \"a\" }",
        "Bitmap { WrapWidth 1, }",
        "Bitmap {",
        "Bitmap {} Bitmap {}",
        "Bitmap { BlendColors, }",
    ] {
        assert!(Texture::decode_mdl(source).is_err(), "{source}");
    }
}
