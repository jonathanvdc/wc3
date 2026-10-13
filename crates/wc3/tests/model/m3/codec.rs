use wc3::model::m3::codec::{Decoder, Encoder};
use wc3::model::m3::{self, Array, Md33, Md34, Ref, VisitReferences as _};

#[derive(
    Clone,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
struct Record {
    value: u16,
    links: [Ref; 2],
}

#[test]
fn derives_select_native_reference_widths() {
    assert_eq!(<Record as m3::RecordLayout<Md33>>::size(), 18);
    assert_eq!(<Record as m3::RecordLayout<Md34>>::size(), 26);
    let wire = [7, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 99, 0, 0, 0];
    let mut input = Decoder::<Md33>::new(&wire);
    let record: Record = input.read().unwrap();
    assert_eq!(record.value, 7);
    assert_eq!(record.links[0], Ref::from_section(2, 3));
    assert_eq!(record.links[1], Ref::from_section(0, 99));
    let mut output = Encoder::<Md33>::new();
    output.write(&record).unwrap();
    assert_eq!(output.finish(), wire);
    let mut wide = Encoder::<Md34>::new();
    wide.write(&record).unwrap();
    assert_eq!(wide.finish().len(), 26);
}

#[test]
fn md33_rejects_flags_and_truncation_reports_origin() {
    let mut output = Encoder::<Md33>::new();
    assert!(output
        .write(&Ref::<m3::Any>::new(1, 0, m3::records::ReferenceFlags(4)))
        .is_err());
    let mut input = Decoder::<Md34>::new(&[0; 11]).with_origin(100);
    let error = input.read::<Ref>().unwrap_err();
    assert_eq!(error.offset, 108);
}

#[test]
fn borrowed_arrays_decode_independent_wire_and_copy_on_edit() {
    let wire = [1, 0, 0, 0, 2, 0, 0, 0];
    let mut array = Array::<u32, Md33>::read(&wire).unwrap();
    assert_eq!(array.as_bytes().as_ptr(), wire.as_ptr());
    assert_eq!(array.get(1).unwrap().unwrap(), 2);
    array.set(0, &99).unwrap();
    assert_eq!(array.get(0).unwrap().unwrap(), 99);
    assert_eq!(wire[0], 1);
    assert!(Array::<u32, Md34>::read(&wire[..7]).is_err());
    assert!(array.set(2, &3).is_err());
}

#[test]
fn derive_visits_every_reference_in_native_order() {
    struct Visitor(Vec<m3::RawReference>);
    impl m3::ReferenceVisitor for Visitor {
        type Error = ();
        fn visit(&mut self, reference: m3::RawReference) -> Result<(), ()> {
            self.0.push(reference);
            Ok(())
        }
    }
    let record = Record {
        value: 9,
        links: [
            Ref::from_section(1, 2),
            Ref::new(0, 3, m3::records::ReferenceFlags(4)),
        ],
    };
    let mut visitor = Visitor(Vec::new());
    record.visit_references(&mut visitor).unwrap();
    assert_eq!(
        visitor.0.iter().map(|r| r.index).collect::<Vec<_>>(),
        [2, 3]
    );
}
