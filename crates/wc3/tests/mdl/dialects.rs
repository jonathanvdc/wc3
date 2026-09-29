use wc3::model::animation::{Animatable, Track};
use wc3::model::geometry::Geoset;
use wc3::model::materials::{Layer, Material};
use wc3::model::mdl::{Dialect, Read as _, Write as _, WriteFields as _, Writer};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::{
    mdl, mdx, DynamicModel, Model, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800,
    V800, V900,
};

const ENGINE: &str = include_str!("../fixtures/mdl/dialects/hd_layer.engine.mdl");
const HIVE: &str = include_str!("../fixtures/mdl/dialects/hd_layer.hive.mdl");
const QUAD: &str = include_str!("../fixtures/mdl/quad_geoset.mdl");
const SLOTS: [&str; 6] = [
    "TextureID",
    "NormalTextureID",
    "ORMTextureID",
    "EmissiveTextureID",
    "TeamColorTextureID",
    "ReflectionsTextureID",
];
fn hive<T: mdl::Write>(value: &T) -> String {
    value
        .encode_mdl_with_dialect(Dialect::HiveWorkshop)
        .unwrap()
}
fn roundtrip_hive<T: mdl::Read + mdl::Write + mdx::Read + mdx::Write>(value: &T) {
    let text = hive(value);
    let decoded = T::decode_mdl(&text).unwrap_or_else(|error| panic!("{error}: {text}"));
    assert_eq!(
        decoded.encode_mdx().unwrap(),
        value.encode_mdx().unwrap(),
        "{text}"
    );
    assert_eq!(hive(&decoded), text);
}
fn geoset_source(fields: &str) -> String {
    let end = QUAD.rfind('}').unwrap();
    format!("{} {fields} }}", &QUAD[..end])
}
#[test]
fn static_slots_match_independent_canonical_fixtures_in_both_dialects() {
    let engine = Layer::<V1800>::decode_mdl(ENGINE).unwrap();
    let hive_layer = Layer::<V1800>::decode_mdl(HIVE).unwrap();
    assert_eq!(engine, hive_layer);
    // Packed independently from the version-1800 MDX Layer specification.
    let binary = include_bytes!("../fixtures/mdl/dialects/hd_layer.mdx");
    assert_eq!(engine.encode_mdx().unwrap(), binary);
    assert_eq!(Layer::<V1800>::decode_mdx(binary).unwrap(), engine);
    assert_eq!(engine.encode_mdl().unwrap(), ENGINE);
    assert_eq!(hive(&engine), HIVE);
    assert_eq!(
        engine.encode_mdx().unwrap(),
        hive_layer.encode_mdx().unwrap()
    );
    roundtrip_hive(&engine);
    let mut writer = Writer::with_dialect(Vec::new(), Dialect::HiveWorkshop);
    assert_eq!(writer.dialect(), Dialect::HiveWorkshop);
    writer.write(&engine).unwrap();
    assert_eq!(writer.finish().unwrap(), HIVE.as_bytes());
}

#[test]
fn sd_bindings_use_subtextures_from_version_1100_in_both_dialects() {
    fn check<V: ModelVersion>() {
        for (shader, name) in [(0, "Shader_SD_Legacy"), (2, "Shader_SD_FixedFunction")] {
            for animated in [false, true] {
                // Pack the documented Layer and SubTexture fields independently
                // of the library writer. KMTF follows the slot, not the layer.
                let mut words = vec![
                    if animated { 92 } else { 68 },
                    0,
                    0,
                    0,
                    u32::MAX,
                    0,
                    1.0f32.to_bits(),
                    1.0f32.to_bits(),
                    1.0f32.to_bits(),
                    1.0f32.to_bits(),
                    1.0f32.to_bits(),
                    0,
                    0,
                    shader,
                    1,
                    if animated { 0 } else { 7 },
                    0,
                ];
                if animated {
                    words.extend([
                        u32::from_le_bytes(*b"KMTF"),
                        1,
                        1,
                        u32::MAX,
                        (-7i32) as u32,
                        7,
                    ]);
                }
                let binary: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
                let decoded = Layer::<V>::decode_mdx(&binary).unwrap();
                assert_eq!(decoded.texture_id, Animatable::Static(0));
                assert!(decoded.alpha.track().is_none());
                let slots = decoded.try_texture_slots().unwrap();
                assert_eq!(slots.len(), 1);
                assert_eq!(slots[0].texture_type, 0);
                assert_eq!(slots[0].texture_id.track().is_some(), animated);

                for dialect in [Dialect::Warcraft3, Dialect::HiveWorkshop] {
                    let binding = if animated {
                        "TextureID 1 { Linear, -7: 7, }"
                    } else if dialect == Dialect::Warcraft3 {
                        "static TextureID 7 <= 0,"
                    } else {
                        "static TextureID 7,"
                    };
                    let shader_field = if dialect == Dialect::Warcraft3 {
                        format!("Shader \"{name}\",")
                    } else {
                        format!("ShaderTypeId {shader},")
                    };
                    // Resolve storage even when the shader follows the binding.
                    let source = format!("Layer {{ {binding} {shader_field} }}");
                    let imported = Layer::<V>::decode_mdl(&source).unwrap();
                    assert_eq!(imported.encode_mdx().unwrap(), binary);
                    let exported = decoded.encode_mdl_with_dialect(dialect).unwrap();
                    assert!(exported.contains(if animated {
                        "TextureID 1"
                    } else if dialect == Dialect::Warcraft3 {
                        "static TextureID 7 <= 0,"
                    } else {
                        "static TextureID 7,"
                    }));
                    assert_eq!(
                        Layer::<V>::decode_mdl(&exported)
                            .unwrap()
                            .encode_mdx()
                            .unwrap(),
                        binary
                    );
                    if shader == 0 {
                        // SD is also the default when no shader is specified.
                        assert_eq!(
                            Layer::<V>::decode_mdl(&format!("Layer {{ {binding} }}"))
                                .unwrap()
                                .encode_mdx()
                                .unwrap(),
                            binary
                        );
                    }
                }
            }
        }
    }
    check::<V1100>();
    check::<V1200>();
    check::<V1300>();
    check::<V1400>();
    check::<V1600>();
    check::<V1800>();
}

#[test]
fn sd_subtexture_export_rejects_hidden_legacy_values() {
    let mut layer = Layer::<V1100>::decode_mdl("Layer { static TextureID 7 <= 0, }").unwrap();
    layer.texture_id = Animatable::Static(3);
    assert!(layer.encode_mdl().is_err());
    layer.texture_id = Animatable::Static(0);
    layer
        .texture_id
        .set_track(Track::<u32>::linear(vec![], None).unwrap());
    assert!(layer.encode_mdl().is_err());
}
#[test]
fn every_named_slot_uses_the_shared_animation_grammar() {
    for interpolation in ["DontInterp", "Linear", "Hermite", "Bezier"] {
        let mut body = String::new();
        for (index, name) in SLOTS.iter().enumerate() {
            let tangents = if matches!(interpolation, "Hermite" | "Bezier") {
                "InTan 7, OutTan 9,"
            } else {
                ""
            };
            body.push_str(&format!(
                "{name} 1 {{ {interpolation}, GlobalSeqId 3, -7: {}, {tangents} }}",
                index + 1
            ));
        }
        // Shader follows bindings: storage is resolved after all fields are read.
        let value =
            Layer::<V1800>::decode_mdl(&format!("Layer {{ {body} ShaderTypeId 1, }}")).unwrap();
        assert_eq!(value.texture_slots().len(), 6);
        assert!(value
            .texture_slots()
            .iter()
            .all(|slot| slot.texture_id.track().is_some()));
        assert!(value.encode_mdl().is_err());
        assert!(value.prepare_mdl_fields(Dialect::Warcraft3).is_err());
        assert!(value.prepare_mdl_fields(Dialect::HiveWorkshop).is_ok());
        roundtrip_hive(&value);
        let decoded = Layer::<V1800>::decode_mdx(&value.encode_mdx().unwrap()).unwrap();
        assert_eq!(hive(&decoded), hive(&value));
        for name in SLOTS {
            assert!(hive(&value).contains(&format!("{name} 1")));
        }
    }
    let mut value =
        Layer::<V1800>::decode_mdl("Layer { ShaderTypeId 1, NormalTextureID 0 { Linear, } }")
            .unwrap();
    let mut slots = value.texture_slots().to_vec();
    slots[0].texture_id.set_value(2);
    value.set_texture_slots(&slots);
    let decoded = Layer::<V1800>::decode_mdl(&hive(&value)).unwrap();
    assert_eq!(decoded.texture_slots()[0].texture_id.value(), Some(&0));
    assert_eq!(
        decoded.texture_slots()[0].texture_id.track(),
        slots[0].texture_id.track()
    );
}
#[test]
fn aliases_share_duplicate_identity_and_error_spans() {
    for body in [
        "ShaderTypeId 1, Shader \"Shader_HD_DefaultUnit\",",
        "Shader \"Shader_HD_DefaultUnit\", ShaderTypeId 1,",
        "static TextureID 1 <= 1, static NormalTextureID 2,",
        "static NormalTextureID 2, static TextureID 1 <= 1,",
        "NormalTextureID 0 { Linear, } static TextureID 1 <= 1,",
        "static NormalTextureID 1, NormalTextureID 0 { Linear, }",
        "NormalTextureID 0 { Linear, } NormalTextureID 0 { Linear, }",
    ] {
        let source = format!("Layer {{ ShaderTypeId 1, {body} }}");
        // Avoid an unrelated duplicate shader in the shader-specific cases.
        let source = if body.starts_with("Shader") {
            format!("Layer {{ {body} }}")
        } else {
            source
        };
        let error = Layer::<V1800>::decode_mdl(&source).unwrap_err();
        assert_eq!(error.kind, mdl::ReadErrorKind::DuplicateField, "{source}");
        assert!(!source[error.span.start..error.span.end].is_empty());
    }
    let error =
        Material::<V800>::decode_mdl("Material { SortPrimsFarZ, SortPrimitives, }").unwrap_err();
    assert_eq!(error.kind, mdl::ReadErrorKind::DuplicateField);
    for body in [
        "static NormalTextureID 1 <= 1,",
        "NormalTextureID 1 { Linear, 0: -1, }",
        "NormalTextureID 2 { Linear, 0: 1, }",
        "static NormalTextureID 1,",
        "ShaderTypeId -1,",
        "ShaderTypeId 4294967296,",
    ] {
        assert!(
            Layer::<V1800>::decode_mdl(&format!("Layer {{ {body} }}")).is_err(),
            "{body}"
        );
    }
}
#[test]
fn numeric_shaders_preserve_unnamed_ids_without_panicking() {
    for id in [0, 1, 2, 24, 42, u32::MAX] {
        let value = Layer::<V1800>::decode_mdl(&format!("Layer {{ ShaderTypeId {id}, }}")).unwrap();
        assert_eq!(value.shader_type().id(), id);
        assert!(hive(&value).contains(&format!("ShaderTypeId {id},")));
        roundtrip_hive(&value);
        assert_eq!(value.encode_mdl().is_ok(), matches!(id, 0 | 1 | 2 | 24));
    }
}
#[test]
fn flags_use_context_specific_spellings_and_omit_engine_only_bits_in_hive() {
    let value = Material::<V800>::decode_mdl(
        "Material { SortPrimitives, FullResolution, Layer { TwoSided, } }",
    )
    .unwrap();
    assert!(value.encode_mdl().unwrap().contains("SortPrimsFarZ,"));
    assert!(hive(&value).contains("SortPrimitives,"));
    roundtrip_hive(&value);
    let material = Material::<V800>::decode_mdl("Material { SortPrimsNearZ, }").unwrap();
    assert!(material.encode_mdl().is_ok());
    assert!(!hive(&material).contains("SortPrimsNearZ"));
    assert_eq!(
        Material::<V800>::decode_mdl(&hive(&material))
            .unwrap()
            .render_mode
            .bits(),
        0
    );
    for flag in [
        "WrapWidth",
        "WrapHeight",
        "Unlit",
        "BackFacesForShadows",
        "AmbientOcclusion",
    ] {
        let layer = Layer::<V800>::decode_mdl(&format!("Layer {{ {flag}, }}")).unwrap();
        assert!(layer.encode_mdl().is_ok());
        assert!(layer.encode_mdl().unwrap().contains(flag));
        assert!(!hive(&layer).contains(flag));
        let restored = Layer::<V800>::decode_mdl(&hive(&layer)).unwrap();
        assert_eq!(restored.shading_flags.bits(), 0);
        let material =
            Material::<V800>::decode_mdl(&format!("Material {{ Layer {{ {flag}, }} }}")).unwrap();
        assert!(!hive(&material).contains(flag));
    }
    let layer = Layer::<V1800>::decode_mdl("Layer { Unshaded, SphereEnvMap, TwoSided, Unfogged, NoDepthTest, NoDepthSet, BackFacesForShadows, AmbientOcclusion, }").unwrap();
    let output = hive(&layer);
    let restored = Layer::<V1800>::decode_mdl(&output).unwrap();
    assert_eq!(
        restored.shading_flags.bits(),
        layer.shading_flags.bits() & !0x70c
    );
    assert_eq!(
        Layer::<V1800>::decode_mdl(&layer.encode_mdl().unwrap()).unwrap(),
        layer
    );
    // The far-Z alias belongs to materials, not emitter flag vocabulary.
    assert!(wc3::model::emitters::ParticleEmitter2::decode_mdl(
        "ParticleEmitter2 \"p\" { ObjectId 0, SortPrimitives, }"
    )
    .is_err());
}
#[test]
fn geoset_dialect_data_and_skin_row_forms_preserve_binary_payloads() {
    let skin = "SkinWeights 4 { { 0, 1, 2, 3, 128, 64, 32, 31 }, { 0, 1, 2, 3, 128, 64, 32, 31 }, { 0, 1, 2, 3, 128, 64, 32, 31 }, { 0, 1, 2, 3, 128, 64, 32, 31 }, }";
    let value = Geoset::<V1400>::decode_mdl(&geoset_source(&format!(
        "{skin} SelectionFlags 128, LevelOfDetail 2, LevelOfDetailName \"LOD2\","
    )))
    .unwrap();
    let text = hive(&value);
    assert!(text.contains("SelectionFlags 128,"));
    assert!(text.contains("LevelOfDetailName \"LOD2\","));
    assert!(text.contains("{ 0, 1, 2, 3, 128, 64, 32, 31 },"));
    assert!(value.encode_mdl().is_err());
    roundtrip_hive(&value);
    let common = Geoset::<V1400>::decode_mdl(&geoset_source(skin)).unwrap();
    let engine = common.encode_mdl().unwrap();
    assert!(engine.contains("\n\t\t0, 1, 2, 3, 128, 64, 32, 31,\n"));
    assert_eq!(
        Geoset::<V1400>::decode_mdl(&engine)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        common.encode_mdx().unwrap()
    );
    roundtrip_hive(&common);
    for bits in [0, 4, 5, u32::MAX] {
        let value =
            Geoset::<V800>::decode_mdl(&geoset_source(&format!("SelectionFlags {bits},"))).unwrap();
        assert_eq!(value.raw_unselectable(), bits);
        roundtrip_hive(&value);
        assert_eq!(value.encode_mdl().is_ok(), matches!(bits, 0 | 4));
        if bits == 4 {
            assert!(hive(&value).contains("Unselectable,"));
        }
    }
}
fn version<V: ModelVersion>() {
    let hd = if V::NUMBER >= 1100 {
        HIVE.to_owned()
    } else {
        "Layer { FilterMode Blend, static TextureID 7, }".to_owned()
    };
    let source = format!("Version {{ FormatVersion {}, }} Model \"Both\" {{}} Materials 1 {{ Material {{ SortPrimitives, {hd} }} }} {QUAD}", V::NUMBER);
    let value = Model::<V>::decode_mdl(&source).unwrap();
    roundtrip_hive(&value);
    let text = hive(&value);
    assert!(text.contains("SortPrimitives,"));
    assert_eq!(
        DynamicModel::decode_mdl(&text)
            .unwrap()
            .encode_mdl_with_dialect(Dialect::HiveWorkshop)
            .unwrap(),
        text
    );
    let engine = value.encode_mdl().unwrap();
    assert_eq!(
        Model::<V>::decode_mdl(&engine)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        value.encode_mdx().unwrap()
    );
}
#[test]
fn typed_and_dynamic_model_io_propagate_dialect_at_every_version() {
    version::<V800>();
    version::<V900>();
    version::<V1000>();
    version::<V1100>();
    version::<V1200>();
    version::<V1300>();
    version::<V1400>();
    version::<V1600>();
    version::<V1800>();
    for source in [
        "Layer { ShaderTypeId 0, }",
        "Layer { ShaderTypeId 1, static NormalTextureID 1, }",
    ] {
        assert!(Layer::<V800>::decode_mdl(source).is_err());
        assert!(Layer::<V900>::decode_mdl(source).is_err());
        assert!(Layer::<V1000>::decode_mdl(source).is_err());
    }
    let material =
        Material::<V900>::decode_mdl("Material { Shader \"Shader_HD_DefaultUnit\", }").unwrap();
    assert_eq!(material.encode_mdl().unwrap(), hive(&material));
    let material =
        Material::<V1000>::decode_mdl("Material { Shader \"Shader_HD_DefaultUnit\", }").unwrap();
    assert_eq!(material.encode_mdl().unwrap(), hive(&material));
}
#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(fields)]
struct AliasedFields {
    #[mdl(property = "EngineValue", hive_name = "ToolValue")]
    value: u32,
    #[mdl(flags(EngineFlag = 1, EngineOnly = 2), hive_flags(ToolFlag = 1))]
    flags: u32,
}
#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Aliases")]
struct AliasedRecord {
    #[mdl(flatten)]
    fields: AliasedFields,
}
#[test]
fn derive_aliases_work_inside_flattened_groups_and_share_presence_bits() {
    let value = AliasedRecord::decode_mdl("Aliases { ToolValue 7, ToolFlag, }").unwrap();
    assert_eq!(
        value.encode_mdl().unwrap(),
        "Aliases {\n\tEngineValue 7,\n\tEngineFlag,\n}\n"
    );
    assert_eq!(hive(&value), "Aliases {\n\tToolValue 7,\n\tToolFlag,\n}\n");
    for fields in [
        "ToolValue 7, EngineValue 7,",
        "EngineValue 7, EngineFlag, ToolFlag,",
    ] {
        assert_eq!(
            AliasedRecord::decode_mdl(&format!("Aliases {{ {fields} }}"))
                .unwrap_err()
                .kind,
            mdl::ReadErrorKind::DuplicateField
        );
    }
    let mut value = AliasedRecord::decode_mdl("Aliases { EngineValue 7, EngineOnly, }").unwrap();
    assert!(hive(&value).contains("EngineOnly,"));
    value.fields.flags = 4; // truly unknown, rather than dialect-specific
    assert!(value
        .encode_mdl_with_dialect(Dialect::HiveWorkshop)
        .is_err());
    let mut writer = Writer::with_dialect(Vec::new(), Dialect::HiveWorkshop);
    assert!(writer.write(&value).is_err());
    assert!(writer.into_inner().is_empty()); // preflight propagates through flatten
}

#[test]
fn whole_model_preserves_hive_only_data_and_rejects_engine_export() {
    let mesh = geoset_source("SelectionFlags 128, LevelOfDetailName \"LOD\",");
    let source = format!("Version {{ FormatVersion 1800, }} Model \"Hive\" {{}} Materials 1 {{ Material {{ SortPrimitives, Layer {{ ShaderTypeId 1, NormalTextureID 1 {{ Bezier, GlobalSeqId 3, -2: 7, InTan 8, OutTan 9, }} }} }} }} {mesh}");
    let model = Model::<V1800>::decode_mdl(&source).unwrap();
    roundtrip_hive(&model);
    assert!(model.encode_mdl().is_err());
    let text = hive(&model);
    let dynamic = DynamicModel::decode_mdl(&text).unwrap();
    assert_eq!(hive(&dynamic), text);
    assert_eq!(dynamic.encode_mdx().unwrap(), model.encode_mdx().unwrap());
    assert_eq!(
        hive(&DynamicModel::decode_mdx(&model.encode_mdx().unwrap(), 1800).unwrap()),
        text
    );
}
