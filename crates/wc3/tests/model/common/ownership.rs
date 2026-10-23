use std::borrow::Cow;
use wc3::model::common::{AsULE, IntoOwned, Ownership, ZeroVec};

#[derive(Debug, IntoOwned)]
struct SemanticRecord<'a, 'label, const N: usize> {
    label: Cow<'label, str>,
    payloads: [Cow<'a, [u8]>; N],
}

#[derive(Debug, IntoOwned)]
enum SemanticTree<'a> {
    Branch(Vec<SemanticTree<'a>>),
    Leaf(Cow<'a, str>),
}

#[test]
fn semantic_detachment_handles_distinct_lifetime_names_and_moves_owned_buffers() {
    let label = "borrowed label".to_owned();
    let owned = vec![1, 2, 3];
    let allocation = owned.as_ptr();
    let detached: SemanticRecord<'static, 'static, 1> = SemanticRecord {
        label: Cow::Borrowed(&label),
        payloads: [Cow::Owned(owned)],
    }
    .into_owned();
    drop(label);
    assert_eq!(detached.label, "borrowed label");
    assert!(matches!(detached.label, Cow::Owned(_)));
    assert_eq!(detached.payloads[0].as_ptr(), allocation);
}

#[test]
fn semantic_detachment_recurses_without_a_self_referential_trait_bound() {
    let value = "leaf".to_owned();
    let detached: SemanticTree<'static> =
        SemanticTree::Branch(vec![SemanticTree::Leaf(Cow::Borrowed(&value))]).into_owned();
    drop(value);
    let SemanticTree::Branch(children) = detached else {
        panic!("branch expected")
    };
    assert!(matches!(&children[0], SemanticTree::Leaf(Cow::Owned(value)) if value == "leaf"));
}

#[derive(Clone, Debug, Ownership)]
struct Record<'data, T: AsULE, const N: usize> {
    values: ZeroVec<'data, T>,
    payloads: [Cow<'data, [u8]>; N],
}

#[derive(Clone, Debug, Ownership)]
enum Entry<'data> {
    Tuple((Cow<'data, [u8]>, ZeroVec<'data, u32>)),
    Named { record: Record<'data, u32, 1> },
    Empty,
}

#[test]
fn format_independent_derive_borrows_and_moves_nested_buffers() {
    let record = Record {
        values: [1u32, 2].into_iter().collect(),
        payloads: [Cow::Owned(vec![3, 4])],
    };
    let values = record.values.as_bytes().as_ptr();
    let payload = record.payloads[0].as_ptr();
    let entry = Entry::Named { record };
    let borrowed = entry.as_borrowed();
    let Entry::Named { record: view } = &borrowed else {
        panic!("named record")
    };
    assert_eq!(view.values.as_bytes().as_ptr(), values);
    assert_eq!(view.payloads[0].as_ptr(), payload);
    let detached: Entry<'static> = borrowed.into_owned();
    let Entry::Named { record: copy } = detached else {
        panic!("named record")
    };
    assert_ne!(copy.values.as_bytes().as_ptr(), values);
    assert_ne!(copy.payloads[0].as_ptr(), payload);
    let Entry::Named { record: moved } = entry.into_owned() else {
        panic!("named record")
    };
    assert_eq!(moved.values.as_bytes().as_ptr(), values);
    assert_eq!(moved.payloads[0].as_ptr(), payload);
}

#[test]
fn tuple_variants_and_unit_variants_share_the_same_derive() {
    let payload = [1, 2];
    let entry = Entry::Tuple((Cow::Borrowed(&payload), [7u32].into_iter().collect()));
    let detached = entry.to_owned();
    let Entry::Tuple((payload, values)) = detached else {
        panic!("tuple record")
    };
    assert!(matches!(payload, Cow::Owned(_)));
    assert_eq!(values.get(0), Some(7));
    assert!(matches!(Entry::Empty.into_owned(), Entry::Empty));
}
