use std::borrow::Cow;
use wc3::blp::{Blp, Blp0, Blp0Content, Blp0Header, Palette};

fn texture<'a>(palette: Palette<'a>, base: Cow<'a, [u8]>, next: Cow<'a, [u8]>) -> Blp<'a> {
    let mut mipmaps = std::array::from_fn(|_| None);
    mipmaps[0] = Some(base);
    mipmaps[1] = Some(next);
    Blp::Blp0(Blp0 {
        header: Blp0Header {
            width: 2,
            height: 1,
            alpha_bits: 0,
            extra: 5,
            has_mipmaps: 1,
        },
        content: Blp0Content::Indexed { palette },
        mipmaps,
    })
}

#[test]
fn editing_borrowed_payload_copies_only_that_payload() {
    let palette = [0; 1024];
    let base = [1, 2];
    let next = [3];
    let mut blp = texture(
        (&palette).into(),
        Cow::Borrowed(&base),
        Cow::Borrowed(&next),
    );
    let Blp::Blp0(value) = &mut blp else {
        unreachable!()
    };
    let mip = value.mipmaps[0].as_mut().unwrap();
    assert_eq!(mip.as_ptr(), base.as_ptr());
    mip.to_mut()[0] = 9;
    let owned_address = mip.as_ptr();
    mip.to_mut()[1] = 8;
    assert_eq!(mip.as_ptr(), owned_address);
    assert_ne!(owned_address, base.as_ptr());
    assert_eq!(base, [1, 2]);
    assert!(matches!(value.mipmaps[1], Some(Cow::Borrowed(_))));
    assert_eq!(value.mipmaps[1].as_ref().unwrap().as_ptr(), next.as_ptr());
    let Blp0Content::Indexed { palette: current } = &value.content else {
        unreachable!()
    };
    assert_eq!(current.as_bytes().as_ptr(), palette.as_ptr());
}

#[test]
fn into_owned_moves_existing_buffers_and_copies_borrowed_payloads() {
    let base = vec![1, 2];
    let base_address = base.as_ptr();
    let palette = Box::new([0; 1024]);
    let palette_address = palette.as_ptr();
    let independent: Blp<'static> = {
        let next = [3];
        let blp = texture(palette.into(), Cow::Owned(base), Cow::Borrowed(&next));
        let view = blp.as_ref();
        let Blp::Blp0(value) = &view else {
            unreachable!()
        };
        assert!(matches!(value.mipmaps[0], Some(Cow::Borrowed(_))));
        assert_eq!(value.mipmaps[0].as_ref().unwrap().as_ptr(), base_address);
        let owned = blp.into_owned();
        let Blp::Blp0(value) = &owned else {
            unreachable!()
        };
        assert_ne!(value.mipmaps[1].as_ref().unwrap().as_ptr(), next.as_ptr());
        owned
    };
    let Blp::Blp0(value) = independent else {
        unreachable!()
    };
    assert_eq!(value.mipmaps[0].as_ref().unwrap().as_ptr(), base_address);
    assert!(value
        .mipmaps
        .iter()
        .flatten()
        .all(|mip| matches!(mip, Cow::Owned(_))));
    let Blp0Content::Indexed { palette } = value.content else {
        unreachable!()
    };
    assert_eq!(palette.as_bytes().as_ptr(), palette_address);
}

#[test]
fn palette_mutation_and_borrowed_views_preserve_fixed_size() {
    let original = [7; 1024];
    let mut palette = Palette::from(&original);
    assert_eq!(palette.as_bytes().as_ptr(), original.as_ptr());
    palette.to_mut()[0] = 9;
    let address = palette.as_bytes().as_ptr();
    assert_ne!(address, original.as_ptr());
    assert_eq!(original[0], 7);
    palette.to_mut()[1] = 8;
    assert_eq!(palette.as_bytes().as_ptr(), address);
    assert_eq!(palette.as_ref().as_bytes().as_ptr(), address);
    assert_eq!(palette.into_owned().as_bytes().as_ptr(), address);
}
