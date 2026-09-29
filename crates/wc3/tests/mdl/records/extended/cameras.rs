use super::*;

#[test]
fn camera_nested_tracks_aliases_required_fields_and_variants() {
    let base = "Camera \"c\" { FieldOfView 1, FarClip 100, ";
    for body in [
        "DOFDistance 1, FocusDistanceKeys 0 { DontInterp, }",
        "FocalLength 1, FocalLength 2,",
        "FStop 1, FStopKeys 0 { DontInterp, }",
        "Target { Translation 0 { DontInterp, } Translation 0 { DontInterp, } }",
        "Target {} Target {}",
        "Rotation 0 { DontInterp, } Rotation 0 { DontInterp, }",
    ] {
        assert!(
            Camera::<V800>::decode_mdl(&format!("{base}{body}}}")).is_err(),
            "{body}"
        );
    }
    assert!(Camera::<V800>::decode_mdl("Camera \"c\" { FarClip 1, }").is_err());
    assert!(Camera::<V800>::decode_mdl("Camera \"c\" { FieldOfView 1, }").is_err());
    let record = Camera::<V800>::decode_mdl(&format!("{base} Visibility 1 {{ Linear, -2: 1, }} Target {{ Position {{ 1, 2, 3 }}, Translation 1 {{ Hermite, GlobalSeqId 4, -5: {{ 1, 2, 3 }}, InTan {{ 4, 5, 6 }}, OutTan {{ 7, 8, 9 }}, }} }} FStop 2.8, FocalLength 50, DOFDistance 180, Rotation 1 {{ Bezier, -3: 1, InTan 2, OutTan 3, }} Translation 1 {{ Linear, -1: {{ 3, 2, 1 }}, }} }}")).unwrap();
    assert!(
        record.translation.is_some()
            && record.rotation.is_some()
            && record.target_translation.is_some()
            && record.visibility.is_some()
            && record.focus_distance.is_some()
            && record.focal_length.is_some()
            && record.f_stop.is_some()
    );
    assert_eq!(
        Camera::<V800>::decode_mdl(&record.encode_mdl().unwrap()).unwrap(),
        record
    );
    assert_eq!(
        Camera::<V800>::decode_mdx(&record.encode_mdx().unwrap()).unwrap(),
        record
    );
    let mut invalid = record;
    invalid.variant = CameraVariant::Variant1([0; 12]);
    assert!(invalid.encode_mdl().is_err());
}
