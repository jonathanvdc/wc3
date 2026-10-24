//! Direct semantic decoding preserves the same native diagnostics as asset loading.
use std::borrow::Cow;
use std::error::Error as _;
use wc3::model::common::FileKey;
use wc3::model::conversion::{self, ImportOptions};
use wc3::model::formats::mdlx::{
    self as native,
    chunks::{ModelChunk, RawChunk, UnknownChunk},
    mdl::{self, Read as _},
    mdx::{self, Write as _},
    DynamicDocument, NoExtensions,
};
use wc3::model::io::{asset, mdlx};

fn reader_error(bytes: &[u8]) -> asset::Error {
    asset::Reader::new(
        asset::Format::Mdlx,
        FileKey::Path("example.mdx".into()),
        bytes,
        ImportOptions::default(),
    )
    .poll()
    .unwrap_err()
}

#[test]
fn mdx_decode_errors_keep_kind_offset_tag_and_source_across_entry_points() {
    for bytes in [
        b"MDLXVERS\x04\x00\x00\x00\x20\x03".as_slice(),
        b"MDLXVERS\x04\x00\x00\x00\xff\xff\x00\x00".as_slice(),
    ] {
        let expected = DynamicDocument::<NoExtensions>::decode_mdx(bytes, 800).unwrap_err();
        assert!(expected.offset > 0);
        let direct = mdlx::decode_mdx(bytes, &ImportOptions::default()).unwrap_err();
        let mdlx::ReadError::Mdx(native) = &direct else {
            panic!("native decoding must precede semantic import");
        };
        assert_eq!(native, &expected);
        assert_eq!(
            direct.source().unwrap().downcast_ref::<mdx::ReadError>(),
            Some(&expected)
        );

        let reader = reader_error(bytes);
        let asset::Error::Decode {
            source: asset::DecodeError::Mdx(native),
            ..
        } = &reader
        else {
            panic!("asset loading must preserve the same binary codec failure");
        };
        assert_eq!(native, &expected);
        assert_eq!(
            reader
                .source()
                .unwrap()
                .source()
                .unwrap()
                .downcast_ref::<mdx::ReadError>(),
            Some(&expected)
        );
    }
}

#[test]
fn mdl_decode_errors_keep_kind_span_and_source_across_entry_points() {
    let source = "Version { FormatVersion 800, }\nModel \"Broken\" { $ }";
    let expected = DynamicDocument::<NoExtensions>::decode_mdl(source).unwrap_err();
    assert_eq!(expected.kind, mdl::ReadErrorKind::InvalidCharacter);
    assert_eq!(&source[expected.span.start..expected.span.end], "$");
    assert_eq!(expected.line_column(source).0, 2);

    let direct = mdlx::decode_mdl(source, &ImportOptions::default()).unwrap_err();
    let mdlx::ReadError::Mdl(native) = &direct else {
        panic!("native text decoding must precede semantic import");
    };
    assert_eq!(native, &expected);
    assert_eq!(
        direct.source().unwrap().downcast_ref::<mdl::ReadError>(),
        Some(&expected)
    );
    assert_eq!(
        native.diagnostic(source).to_string(),
        expected.diagnostic(source).to_string()
    );

    let reader = reader_error(source.as_bytes());
    let asset::Error::Decode {
        source: asset::DecodeError::Mdl(native),
        ..
    } = &reader
    else {
        panic!("asset loading must preserve the same text codec failure");
    };
    assert_eq!(native, &expected);
    assert_eq!(
        reader
            .source()
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<mdl::ReadError>(),
        Some(&expected)
    );
}

#[test]
fn valid_mdl_with_invalid_graph_reports_import_phase_and_semantic_path() {
    let source = r#"Version { FormatVersion 800, } Model "Duplicate nodes" {}
        Helper "First" { ObjectId 0, }
        Helper "Second" { ObjectId 0, }
    "#;
    let document = native::Document::<native::V800>::decode_mdl(source).unwrap();
    let expected = mdlx::import_owned(&document, &ImportOptions::default()).unwrap_err();
    assert_eq!(expected.path, "nodes");

    let direct = mdlx::decode_mdl(source, &ImportOptions::default()).unwrap_err();
    let mdlx::ReadError::Import(import) = &direct else {
        panic!("a decoded invalid graph must fail during semantic import");
    };
    assert_eq!(import, &expected);
    assert_eq!(
        direct.source().unwrap().downcast_ref::<conversion::Error>(),
        Some(&expected)
    );
    let asset::Error::Import { source: import, .. } = reader_error(source.as_bytes()) else {
        panic!("asset loading must retain the semantic phase");
    };
    assert_eq!(import, expected);
}

#[test]
fn valid_mdx_with_rejected_unknown_data_reports_import_phase() {
    let mut document =
        native::Document::<native::V800>::decode_mdl(include_str!("fixtures/mdl/quad_model.mdl"))
            .unwrap();
    document.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk::new(
            *b"TEST",
            Cow::Borrowed([1u8, 2, 3].as_slice()),
        ))
        .unwrap(),
    ));
    let bytes = document.encode_mdx().unwrap();
    let expected = mdlx::import_owned(&document, &ImportOptions::default()).unwrap_err();
    assert_eq!(expected.path, "chunks");

    let direct = mdlx::decode_mdx(&bytes, &ImportOptions::default()).unwrap_err();
    let mdlx::ReadError::Import(import) = &direct else {
        panic!("unknown native data is decoded before import policy rejects it");
    };
    assert_eq!(import, &expected);
    assert_eq!(
        direct.source().unwrap().downcast_ref::<conversion::Error>(),
        Some(&expected)
    );
    let asset::Error::Import { source: import, .. } = reader_error(&bytes) else {
        panic!("asset loading must retain the semantic phase");
    };
    assert_eq!(import, expected);
}
