use wc3::model::formats::pkb::{Argument, Document, Instruction, Register, Script};

const MAGIC: [u8; 4] = [0x11, 0x0b, 0x00, 0xca];
const HEADER_SIZE: usize = 28;

fn fixture(name: &[u8]) -> Vec<u8> {
    let body = [0xa0, 0, 0, 0, 0, 1, 0, 0xff, 0xff, 1, 2, 3, 4];
    let mut bytes = Vec::new();
    bytes.extend(MAGIC);
    bytes.extend([2, 5, 1, 1]);
    for word in [
        17u32,
        1,
        1,
        (HEADER_SIZE + 8 + 4 + body.len()) as u32,
        0x12345678,
    ] {
        bytes.extend(word.to_le_bytes());
    }
    bytes.extend(0u32.to_le_bytes());
    bytes.extend(0xdeadbeefu32.to_le_bytes());
    bytes.extend((body.len() as u32).to_le_bytes());
    bytes.extend(body);
    bytes.extend(2u32.to_le_bytes());
    if name.len() < 128 {
        bytes.push(name.len() as u8);
    } else {
        bytes.extend([0x80 | ((name.len() >> 8) as u8), name.len() as u8]);
    }
    bytes.extend(name);
    bytes.push(0);
    bytes
}

#[test]
fn preserves_unknown_payloads_flags_metadata_and_non_utf8_strings() {
    let bytes = fixture(b"Unknown\xffObject");
    let document = Document::read(&bytes).unwrap();
    assert_eq!(document.as_bytes(), bytes);
    assert_eq!(document.header().reserved, 0x12345678);
    assert_eq!(document.types()[0].reserved, 0xdeadbeef);
    assert_eq!(document.strings(), &[b"Unknown\xffObject".as_slice(), b""]);
    let object = document.object(1).unwrap();
    assert_eq!(object.flags, 0xa0);
    assert_eq!(object.field_count, 1);
    assert_eq!(object.fields, [0xff, 0xff, 1, 2, 3, 4]);
    assert!(document.object(0).is_none());
    assert!(document.object(2).is_none());
    assert_eq!(object.fields.as_ptr(), bytes[47..].as_ptr());
}

#[test]
fn decodes_high_part_first_long_string_lengths() {
    let name = vec![b'x'; 517];
    let bytes = fixture(&name);
    let document = Document::read(&bytes).unwrap();
    assert_eq!(document.strings()[0], name);
    assert!(document.strings()[1].is_empty());
    assert_eq!(
        Document::read(&fixture(&[])).unwrap().strings(),
        &[b"".as_slice(), b""]
    );
}

#[test]
fn rejects_every_truncated_prefix_and_extra_trailing_bytes() {
    let bytes = fixture(&vec![b'x'; 517]);
    for end in 0..bytes.len() {
        assert!(Document::read(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut extra = bytes;
    extra.push(0);
    assert!(Document::read(&extra).is_err());
}

#[test]
fn rejects_overlaps_counts_and_invalid_type_names_without_allocating_them() {
    let bytes = fixture(b"T");
    for (offset, word) in [
        (12, u32::MAX),
        (16, u32::MAX),
        (20, 0),
        (20, u32::MAX),
        (28, 2),
        (36, u32::MAX),
        (41, 1),
    ] {
        let mut invalid = bytes.clone();
        invalid[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
        assert!(Document::read(&invalid).is_err(), "word at {offset}");
    }
    let mut invalid = bytes;
    invalid[5] = 9;
    assert!(Document::read(&invalid).is_err());
}

fn script_bytes(constants: &[[u32; 8]], code: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for word in [
        0u32,
        0,
        (constants.len() * 32) as u32,
        code.len() as u32,
        0,
        2,
        3,
        4,
        5,
    ] {
        bytes.extend(word.to_le_bytes());
    }
    for slot in constants {
        for word in slot {
            bytes.extend(word.to_le_bytes());
        }
    }
    bytes.extend(code);
    bytes.resize(bytes.len().next_multiple_of(4), 0);
    bytes
}

#[test]
fn script_constants_keep_thirty_two_byte_stride_and_metadata() {
    let slots = [
        [1, 2, 3, 4, 0xdeadbeef, 6, 7, 8],
        [11, 12, 13, 14, 15, 16, 17, 18],
    ];
    let bytes = script_bytes(&slots, &[]);
    let script = Script::read(&bytes).unwrap();
    assert_eq!(script.constant(0), Some([1, 2, 3, 4]));
    assert_eq!(script.constant(1), Some([11, 12, 13, 14]));
    assert_eq!(script.constant(2), None);
    assert_eq!(script.constants, &bytes[36..100]);
    assert_eq!(script.register_counts, [0, 2, 3, 4, 5]);
}

#[test]
fn script_decoding_preserves_scopes_signed_receivers_and_argument_flags() {
    let mut code = vec![0x43];
    code.extend(0x2303_1234u32.to_le_bytes());
    code.extend(3u16.to_le_bytes());
    code.push(0x52);
    for register in [0x20010000u32, 0x20000001, 0x20000002, 0x02010003] {
        code.extend(register.to_le_bytes());
    }
    code.extend([0x53, 9]);
    code.extend((-1i16).to_le_bytes());
    code.extend(7u16.to_le_bytes());
    code.push(2);
    code.extend(u32::MAX.to_le_bytes());
    for (flags, register) in [(1u8, 0x20010002u32), (2, 0x1a010003)] {
        code.push(flags);
        code.extend(register.to_le_bytes());
    }
    let bytes = script_bytes(&[], &code);
    let script = Script::read(&bytes).unwrap();
    assert_eq!(
        script
            .instructions
            .iter()
            .map(|i| i.offset)
            .collect::<Vec<_>>(),
        [0, 7, 24]
    );
    assert_eq!(
        script.instructions[1].instruction,
        Instruction::Select {
            destination: Register(0x20010000),
            if_false: Register(0x20000001),
            if_true: Register(0x20000002),
            condition: Register(0x02010003),
        }
    );
    let Instruction::Call {
        flags,
        receiver,
        function,
        result,
        arguments,
    } = &script.instructions[2].instruction
    else {
        panic!("expected a call");
    };
    assert_eq!(
        (*flags, *receiver, *function, *result),
        (9, -1, 7, Register::VOID)
    );
    assert_eq!(
        arguments[1],
        Argument {
            flags: 2,
            register: Register(0x1a010003)
        }
    );
    let register = Register(0x2303_1234);
    assert_eq!(
        (register.bank(), register.scope(), register.slot()),
        (0x23, 3, 0x1234)
    );
}

#[test]
fn malformed_script_regions_and_instruction_operands_fail_explicitly() {
    for code in [
        vec![0x43],
        vec![0x4c, 255],
        vec![0x69],
        vec![0x53, 0, 255, 255, 0, 0, 255, 255, 255, 255, 255],
    ] {
        assert!(Script::read(&script_bytes(&[], &code)).is_err());
    }
    let mut bytes = script_bytes(&[], &[]);
    bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
    assert!(Script::read(&bytes).is_err());
    let bytes = script_bytes(&[[0; 8]], &[0x44, 0, 0, 0, 0, 0, 0]);
    for end in 0..bytes.len() - 3 {
        assert!(Script::read(&bytes[..end]).is_err(), "prefix {end}");
    }
}

fn blob_fixture(fields: &[u8], field_count: u16) -> Vec<u8> {
    let original = fixture(b"CCompilerBlobCache");
    let string_offset = u32::from_le_bytes(original[20..24].try_into().unwrap()) as usize;
    let mut bytes = original[..36].to_vec();
    bytes.extend(((7 + fields.len()) as u32).to_le_bytes());
    bytes.extend([0x20, 0, 0, 0, 0]);
    bytes.extend(field_count.to_le_bytes());
    bytes.extend(fields);
    let offset = bytes.len() as u32;
    bytes[20..24].copy_from_slice(&offset.to_le_bytes());
    bytes.extend(&original[string_offset..]);
    bytes
}

#[test]
fn typed_compiler_cache_borrows_its_word_array_and_preserves_stage_identity() {
    let script = script_bytes(&[[0; 8]], &[]);
    let mut fields = Vec::new();
    fields.extend(1u16.to_le_bytes());
    fields.extend(5u32.to_le_bytes());
    fields.extend(2u16.to_le_bytes());
    fields.extend(((script.len() / 4) as u32).to_le_bytes());
    fields.extend(&script);
    fields.extend(5u16.to_le_bytes());
    fields.extend(1u32.to_le_bytes());
    let bytes = blob_fixture(&fields, 3);
    let document = Document::read(&bytes).unwrap();
    let blob = document.compiler_blobs().next().unwrap().unwrap();
    assert_eq!(blob.identifier, 5);
    assert_eq!(blob.entrypoint, 1);
    assert_eq!(blob.bytes, script);
    assert_eq!(blob.bytes.as_ptr(), bytes[59..].as_ptr());
    assert_eq!(blob.script().unwrap().constant(0), Some([0; 4]));
}

#[test]
fn unknown_duplicate_or_invalid_cache_fields_leave_native_document_intact() {
    let cases = [
        (vec![6, 0, 0, 0, 0, 0], 1),
        (vec![1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0], 2),
        (vec![3, 0, 1, 0, 0, 0, 2, 0, 0, 0], 1),
        (vec![2, 0, 255, 255, 255, 255], 1),
    ];
    for (fields, count) in cases {
        let bytes = blob_fixture(&fields, count);
        let document = Document::read(&bytes).unwrap();
        assert!(document.compiler_blobs().next().unwrap().is_err());
        assert_eq!(document.as_bytes(), bytes);
    }
}
