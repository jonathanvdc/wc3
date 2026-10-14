use super::shuffle;

// Distinct lane labels make these checks independent of the traversal.
#[test]
fn documented_two_block_planes() {
    let examples: &[(u32, &[u8], &[u8])] = &[
        (9, b"AAAAbbbbCCCCdddd", b"AAAACCCCbbbbdddd"),
        (10, b"AAAAbbbbCCCCdddd", b"AAAACCCCbbbbdddd"),
        (
            11,
            b"AAAAAAAA1111aaaaBBBBBBBB2222bbbb",
            b"AAAAAAAABBBBBBBB11112222aaaabbbb",
        ),
        (
            12,
            b"AA111111bbbbccccDD222222eeeeffff",
            b"AADD111111222222bbbbeeeeccccffff",
        ),
        (
            43,
            b"AA111111bbbbccccDD222222eeeeffff",
            b"AADD111111222222bbbbeeeeccccffff",
        ),
    ];
    for &(format, blocks, planes) in examples {
        assert_eq!(shuffle(format, blocks, true).unwrap(), planes);
        assert_eq!(shuffle(format, planes, false).unwrap(), blocks);
    }
}

#[test]
fn boundaries_and_passthrough() {
    for format in [9, 10, 11, 12, 43] {
        assert!(shuffle(format, &[], true).unwrap().is_empty());
        assert!(shuffle(format, &[0; 7], false).is_err());
    }
    assert_eq!(
        shuffle(999, b"uncompressed", true).unwrap(),
        b"uncompressed"
    );
}
