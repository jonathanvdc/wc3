use wc3::model::geometry::{Geoset, SkinWeights};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::{mdl, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900};

const QUAD: &str = include_str!("../../fixtures/mdl/quad_geoset.mdl");
const CANONICAL: &str = include_str!("../../fixtures/mdl/quad_geoset.canonical.mdl");
const EMPTY: &str =
    "Geoset { Vertices 0 {} Normals 0 {} VertexGroup {} Faces 0 0 {} Groups 0 0 {} }";
fn with_fields(source: &str, fields: &str) -> String {
    let end = source.rfind('}').unwrap();
    format!("{} {fields} }}", &source[..end])
}
fn roundtrip<V: ModelVersion>(value: &Geoset<V>) {
    let binary = value.encode_mdx().unwrap();
    let text = value.encode_mdl().unwrap();
    let decoded = Geoset::<V>::decode_mdl(&text).unwrap_or_else(|error| panic!("{error}: {text}"));
    assert_eq!(decoded.encode_mdx().unwrap(), binary);
    assert_eq!(
        Geoset::<V>::decode_mdx(&binary)
            .unwrap()
            .encode_mdl()
            .unwrap(),
        text
    );
}
#[test]
fn specification_quad_has_independent_canonical_output_and_wire_sections() {
    let value = Geoset::<V800>::decode_mdl(QUAD).unwrap();
    assert_eq!(
        value.vertices(),
        [
            [-16.0, -16.0, 0.0],
            [16.0, -16.0, 0.0],
            [16.0, 16.0, 0.0],
            [-16.0, 16.0, 0.0]
        ]
    );
    assert_eq!(value.face_indices(), [0, 1, 2, 0, 2, 3]);
    assert_eq!(value.primitive_types(), [4]);
    assert_eq!(value.primitive_counts(), [6]);
    assert_eq!(value.matrix_group_sizes(), [1]);
    assert_eq!(value.matrix_indices(), [0]);
    assert_eq!(
        value.uv_sets()[0],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]
    );
    assert_eq!(value.encode_mdl().unwrap(), CANONICAL);
    // Independently assembled from the companion MDX section layout.
    assert_eq!(
        value.encode_mdx().unwrap(),
        include_bytes!("../../fixtures/mdl/quad_geoset.mdx")
    );
    roundtrip(&value);
}
#[test]
fn all_versions_defaults_and_multiple_mesh_groups_roundtrip() {
    macro_rules! check { ($($version:ty),*) => { $( {
        let value = Geoset::<$version>::decode_mdl(QUAD).unwrap();
        assert_eq!(value.material_id, 0);
        assert_eq!(value.selection_group, 0);
        assert_eq!(value.raw_unselectable(), 0);
        roundtrip(&value);
        roundtrip(&Geoset::<$version>::decode_mdl(EMPTY).unwrap());
    } )* }; }
    check!(V800, V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800);
    let source = QUAD
        .replace("Faces 1 6 {", "Faces 2 6 {")
        .replace(
            "{ 0, 1, 2, 0, 2, 3 },",
            "{ 0, 1, 2 }, } Triangles { { 0, 2, 3 },",
        )
        .replace("Groups 1 1 {", "Groups 3 3 {")
        .replace(
            "Matrices { 0 },",
            "Matrices { 0, 1 }, Matrices {}, Matrices { 2 },",
        );
    let source = with_fields(&source, "TVertices 4 { { -0.0, 1.0 }, { 1.0, 1.0 }, { 1.0, 0.0 }, { 0.0, 0.0 }, } Anim { BoundsRadius -0.0, } Anim { MaximumExtent { 1.0, 2.0, 3.0 }, } Unselectable,");
    let value = Geoset::<V900>::decode_mdl(&source).unwrap();
    assert_eq!(value.primitive_counts(), [3, 3]);
    assert_eq!(value.matrix_group_sizes(), [2, 0, 1]);
    assert_eq!(value.uv_sets().len(), 2);
    assert_eq!(value.sequence_extents.len(), 2);
    assert_eq!(
        value.sequence_extents[0].bounds_radius.to_bits(),
        (-0.0f32).to_bits()
    );
    assert_eq!(value.raw_unselectable(), 4);
    roundtrip(&value);
}
#[test]
fn version_selected_sections_accept_both_skin_row_forms_and_keep_empty_presence() {
    let bare = "0, 1, 2, 255, 255, 0, 0, 0,";
    let braced = "{ 0, 1, 2, 255, 255, 0, 0, 0 },";
    for row in [bare, braced] {
        let fields = format!(
            "LevelOfDetail 2, Tangents 4 {{ {} }} SkinWeights 4 {{ {} }}",
            "{ 1.0, 0.0, 0.0, -1.0 },".repeat(4),
            row.repeat(4)
        );
        let source = with_fields(QUAD, &fields);
        assert_eq!(
            Geoset::<V800>::decode_mdl(&source).unwrap_err().kind,
            mdl::ReadErrorKind::UnsupportedField
        );
        macro_rules! check { ($($version:ty),*) => { $( {
            let value = Geoset::<$version>::decode_mdl(&source).unwrap();
            assert_eq!(value.level_of_detail(), 2);
            assert_eq!(value.skin_weights().unwrap()[0].bone_indices, [0, 1, 2, 255]);
            assert_eq!(value.tangents().unwrap()[0], [1.0, 0.0, 0.0, -1.0]);
            assert!(value.encode_mdl().unwrap().contains(bare));
            roundtrip(&value);
        } )* }; }
        check!(V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800);
    }
    for field in [
        "LevelOfDetail 0,",
        "LevelOfDetailName \"\",",
        "Tangents 0 {}",
        "SkinWeights 0 {}",
    ] {
        let source = with_fields(EMPTY, field);
        assert_eq!(
            Geoset::<V800>::decode_mdl(&source).unwrap_err().kind,
            mdl::ReadErrorKind::UnsupportedField
        );
        roundtrip(&Geoset::<V900>::decode_mdl(&source).unwrap());
    }
    let value =
        Geoset::<V900>::decode_mdl(&with_fields(EMPTY, "SkinWeights 0 {} Tangents 0 {}")).unwrap();
    assert_eq!(value.skin_weights(), Some([].as_slice()));
    assert_eq!(value.tangents(), Some([].as_slice()));
    roundtrip(&value);
}
#[test]
fn malformed_counts_duplicates_ranges_and_unknown_fields_have_spans() {
    let cases = [
        QUAD.replace("Vertices 4", "Vertices 3"),
        QUAD.replace("Normals 4", "Normals 5"),
        QUAD.replace("TVertices 4", "TVertices 5"),
        QUAD.replace("Faces 1 6", "Faces 2 6"),
        QUAD.replace("Faces 1 6", "Faces 1 7"),
        QUAD.replace("Groups 1 1", "Groups 2 1"),
        QUAD.replace("Groups 1 1", "Groups 1 2"),
        QUAD.replace("{ 0, 1, 2, 0, 2, 3 }", "{ 0, 1 }"),
        QUAD.replace("{ 0, 1, 2, 0, 2, 3 }", "{ 0, 1, 65536 }"),
        with_fields(QUAD, "Vertices 0 {}"),
        with_fields(QUAD, "Faces 0 0 {}"),
        with_fields(QUAD, "Unselectable, Unselectable,"),
        with_fields(QUAD, "Unselectable, SelectionFlags 0,"),
        with_fields(QUAD, "SelectionFlags 0, SelectionFlags 4,"),
        with_fields(QUAD, "LevelOfDetail 0, LevelOfDetail 0,"),
        with_fields(QUAD, "Tangents 0 {} Tangents 0 {}"),
        with_fields(QUAD, "SkinWeights 0 {} SkinWeights 0 {}"),
        with_fields(QUAD, "ComponentSkin 0 {}"),
        with_fields(QUAD, "SkinWeights 4 { 256, 0, 0, 0, 255, 0, 0, 0, }"),
        QUAD.replace("Normals 4", "Normals 0")
            .replace("\t\t{ 0.0, 0.0, 1.0 },\n", ""),
        "Geoset {}".to_owned(),
    ];
    for source in cases {
        let error = Geoset::<V900>::decode_mdl(&source).unwrap_err();
        assert!(error.span.end <= source.len(), "{error}: {source}");
    }
    for field in ["Tangents 0 {}", "SkinWeights 0 {}"] {
        assert!(Geoset::<V900>::decode_mdl(&with_fields(QUAD, field)).is_err());
    }
}
#[test]
fn binary_only_values_are_rejected_instead_of_dropped() {
    let mut value = Geoset::<V1400>::decode_mdl(QUAD).unwrap();
    value
        .set_skin_weights(Some(
            &[SkinWeights {
                bone_indices: [256, 0, 0, 0],
                weights: [255, 0, 0, 0],
            }; 4],
        ))
        .unwrap();
    assert!(value.encode_mdl().is_err());
    let value =
        Geoset::<V900>::decode_mdl(&with_fields(QUAD, "LevelOfDetailName \"Mesh\",")).unwrap();
    assert_eq!(value.name(), "Mesh");
    assert!(value.encode_mdl().unwrap().contains("Name \"Mesh\","));
    roundtrip(&value);
    let value = Geoset::<V800>::decode_mdl(&with_fields(QUAD, "SelectionFlags 128,")).unwrap();
    assert_eq!(value.raw_unselectable(), 128);
    assert!(value.encode_mdl().is_err());
    for (tag, word) in [(*b"PTYP", 5u32), (*b"PCNT", 3), (*b"MTGC", 2)] {
        let mut binary = Geoset::<V800>::decode_mdl(QUAD)
            .unwrap()
            .encode_mdx()
            .unwrap();
        let offset = binary.windows(4).position(|bytes| bytes == tag).unwrap() + 8;
        binary[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
        assert!(Geoset::<V800>::decode_mdx(&binary)
            .unwrap()
            .encode_mdl()
            .is_err());
    }
}

#[test]
fn fields_reconstruct_independently_of_source_order() {
    let value = Geoset::<V900>::decode_mdl(
        r#"Geoset {
        SkinWeights 1 { { 0, 0, 0, 0, 255, 0, 0, 0 }, }
        Groups 1 1 { Matrices { 0 }, }
        MaterialID 7,
        Anim { MinimumExtent { -0.0, 2.0, 3.0 }, }
        Normals 1 { { 0.0, 0.0, 1.0 }, }
        TVertices 1 { { 0.0, 1.0 }, }
        Faces 1 3 { Triangles { { 0, 0, 0 }, } }
        Tangents 1 { { 1.0, 0.0, 0.0, -1.0 }, }
        VertexGroup { 0, }
        LevelOfDetail 3,
        SelectionFlags 4,
        Vertices 1 { { -0.0, 2.0, 3.0 }, }
    }"#,
    )
    .unwrap();
    assert_eq!(value.material_id, 7);
    assert_eq!(value.selection_group, 0);
    assert_eq!(value.level_of_detail(), 3);
    assert_eq!(value.extent.bounds_radius.to_bits(), 0.0f32.to_bits());
    assert_eq!(value.vertices()[0][0].to_bits(), (-0.0f32).to_bits());
    roundtrip(&value);
}

#[test]
fn selection_accessors_use_the_text_mask_and_preserve_other_binary_bits() {
    let mut value = Geoset::<V800>::decode_mdl(QUAD).unwrap();
    value.set_unselectable(true);
    assert_eq!(value.raw_unselectable(), 4);
    assert!(value.unselectable());
    roundtrip(&value);
    value.set_raw_unselectable(128);
    assert!(!value.unselectable());
    value.set_unselectable(true);
    assert_eq!(value.raw_unselectable(), 132);
    value.set_unselectable(false);
    assert_eq!(value.raw_unselectable(), 128);
    assert!(value.encode_mdl().is_err());
}

#[test]
fn skinned_geosets_preserve_empty_legacy_vertex_groups() {
    // Scarlet Footman and Highborn Vashj use SKIN with an empty GNDX array.
    // Reduce that layout to the existing four-vertex synthetic quad.
    let source = with_fields(
        QUAD,
        &format!(
            "SkinWeights 4 {{ {} }}",
            "0, 0, 0, 0, 255, 0, 0, 0,".repeat(4)
        ),
    );
    macro_rules! check {
        ($($version:ty),*) => { $( {
            let mut binary = Geoset::<$version>::decode_mdl(&source).unwrap().encode_mdx().unwrap();
            let group = binary.windows(4).position(|tag| tag == b"GNDX").unwrap();
            binary[group + 4..group + 8].copy_from_slice(&0u32.to_le_bytes());
            binary.drain(group + 8..group + 12);
            let length = binary.len() as u32;
            binary[..4].copy_from_slice(&length.to_le_bytes());
            let geoset = Geoset::<$version>::decode_mdx(&binary).unwrap();
            assert!(geoset.vertex_groups().is_empty());
            for dialect in [mdl::Dialect::Warcraft3, mdl::Dialect::HiveWorkshop] {
                let text = geoset.encode_mdl_with_dialect(dialect).unwrap();
                let decoded = Geoset::<$version>::decode_mdl(&text).unwrap();
                assert!(decoded.vertex_groups().is_empty());
                assert_eq!(decoded.skin_weights(), geoset.skin_weights());
                assert_eq!(decoded.encode_mdx().unwrap(), binary);
                // Independent writers may omit the block entirely.
                let start = text.find("\tVertexGroup {\n").unwrap();
                let end = start + text[start..].find("\t}\n").unwrap() + 3;
                let omitted = format!("{}{}", &text[..start], &text[end..]);
                assert_eq!(Geoset::<$version>::decode_mdl(&omitted).unwrap().encode_mdx().unwrap(), binary);
            }
        } )* };
    }
    check!(V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800);
}

#[test]
fn skin_weights_only_replace_an_empty_legacy_group_array() {
    let start = QUAD.find("\tVertexGroup {\n").unwrap();
    let end = start + QUAD[start..].find("\t}\n").unwrap() + 3;
    let omitted = format!("{}{}", &QUAD[..start], &QUAD[end..]);
    let skin = format!(
        "SkinWeights 4 {{ {} }}",
        "0, 0, 0, 0, 255, 0, 0, 0,".repeat(4)
    );
    let valid = with_fields(&omitted, &skin);
    assert!(Geoset::<V900>::decode_mdl(&valid).is_ok());
    assert!(Geoset::<V900>::decode_mdl(&omitted).is_err());
    assert!(Geoset::<V800>::decode_mdl(&omitted).is_err());
    for fields in ["SkinWeights 0 {}", "VertexGroup { 0, }"] {
        let source = if fields.starts_with("VertexGroup") {
            with_fields(&valid, fields)
        } else {
            with_fields(&omitted, fields)
        };
        assert!(Geoset::<V900>::decode_mdl(&source).is_err());
    }
    // Complete skinning does not excuse missing normals or UVs.
    for block in ["Normals", "TVertices"] {
        let start = valid.find(&format!("\t{block} 4 {{\n")).unwrap();
        let end = start + valid[start..].find("\t}\n").unwrap() + 3;
        let invalid = format!("{}\t{block} 0 {{}}\n{}", &valid[..start], &valid[end..]);
        assert!(Geoset::<V900>::decode_mdl(&invalid).is_err());
    }
    // Likewise, malformed nonempty GNDX remains unrepresentable on writing.
    let mut binary = Geoset::<V900>::decode_mdl(&with_fields(QUAD, &skin))
        .unwrap()
        .encode_mdx()
        .unwrap();
    let group = binary.windows(4).position(|tag| tag == b"GNDX").unwrap();
    binary[group + 4..group + 8].copy_from_slice(&1u32.to_le_bytes());
    binary.drain(group + 9..group + 12);
    let length = binary.len() as u32;
    binary[..4].copy_from_slice(&length.to_le_bytes());
    assert!(Geoset::<V900>::decode_mdx(&binary)
        .unwrap()
        .encode_mdl()
        .is_err());
}

#[test]
fn geoset_names_use_dialect_aliases_and_preserve_binary_storage() {
    use wc3::model::mdl::Dialect;
    macro_rules! check {
        ($($version:ty),*) => { $( {
            for spelling in ["Name", "LevelOfDetailName"] {
                let value = Geoset::<$version>::decode_mdl(&with_fields(QUAD,
                    &format!("{spelling} \"LOD mesh\","))).unwrap();
                let original = value.encode_mdx().unwrap();
                for (dialect, expected) in [(Dialect::Warcraft3, "Name"),
                    (Dialect::HiveWorkshop, "LevelOfDetailName")] {
                    let text = value.encode_mdl_with_dialect(dialect).unwrap();
                    assert!(text.contains(&format!("\n\t{expected} \"LOD mesh\",")));
                    assert_eq!(Geoset::<$version>::decode_mdl(&text).unwrap().encode_mdx().unwrap(), original);
                }
            }
        } )* };
    }
    check!(V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800);
    for spelling in ["Name", "LevelOfDetailName"] {
        assert!(
            Geoset::<V800>::decode_mdl(&with_fields(QUAD, &format!("{spelling} \"Mesh\",")))
                .is_err()
        );
    }
    for fields in [
        "Name \"A\", LevelOfDetailName \"B\",",
        "LevelOfDetailName \"A\", Name \"B\",",
    ] {
        let error = Geoset::<V900>::decode_mdl(&with_fields(QUAD, fields)).unwrap_err();
        assert_eq!(error.kind, mdl::ReadErrorKind::DuplicateField);
    }
}
