//! Validation and array traversal derives compose without codec trait bounds.
use m2::codec::{Array, Value};
use m2::records::{ModernTimeline, Track, TrackBase, TrackValues};
use m2::{ReadError, Validate as _, VisitArrays as _, ZeroVec, V264};
use std::cell::RefCell;
use std::rc::Rc;
use wc3::model::m2;
use wc3::model::m2::records::Interpolation;
use wc3::model::m2::records::ModelFlags;

struct Leaf<'data> {
    id: usize,
    values: Array<'data, u32>,
    validation: Rc<RefCell<Vec<usize>>>,
    invalid: bool,
}
impl m2::Validate for Leaf<'_> {
    fn validate(&self) -> Result<(), ReadError> {
        self.validation.borrow_mut().push(self.id);
        if self.invalid {
            Err(ReadError::invalid(self.id, "test leaf"))
        } else {
            Ok(())
        }
    }
}
impl<'data> m2::VisitArrays<'data> for Leaf<'data> {
    fn visit_arrays<V: m2::ArrayVisitor<'data>>(
        &mut self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        visitor.visit(self.id, u16::MAX, &mut self.values)
    }
}
fn leaf<'data>(id: usize, validation: &Rc<RefCell<Vec<usize>>>) -> Leaf<'data> {
    Leaf {
        id,
        values: Array::Inline(ZeroVec::alloc_from_slice(&[id as u32])),
        validation: validation.clone(),
        invalid: false,
    }
}
#[derive(m2::Validate, m2::VisitArrays)]
struct Named<'data, V> {
    first: Leaf<'data>,
    generic: V,
    last: Option<Box<Leaf<'data>>>,
}
#[derive(m2::Validate, m2::VisitArrays)]
struct Tuple<T>(T, [T; 1]);
#[derive(m2::Validate, m2::VisitArrays)]
struct Unit;
#[derive(m2::Validate, m2::VisitArrays)]
enum Choice<'data, V> {
    Named { first: Leaf<'data>, second: V },
    Tuple(Leaf<'data>, V),
    Unit,
}
struct Visitor {
    seen: Vec<(usize, u16)>,
    fail_at: Option<usize>,
}
impl<'data> m2::ArrayVisitor<'data> for Visitor {
    type Error = usize;
    fn visit<T: Value>(
        &mut self,
        sequence: usize,
        global: u16,
        values: &mut Array<'data, T>,
    ) -> Result<(), usize> {
        self.seen.push((sequence, global));
        if self.fail_at == Some(self.seen.len()) {
            return Err(sequence);
        }
        // Mutation proves the derive passes the original mutable leaf.
        *values = Array::Inline(ZeroVec::new());
        Ok(())
    }
}
fn visitor() -> Visitor {
    Visitor {
        seen: Vec::new(),
        fail_at: None,
    }
}
#[test]
fn structs_preserve_order_mutate_and_stop_at_the_first_error() {
    let validation = Rc::new(RefCell::new(Vec::new()));
    let mut value = Named {
        first: leaf(1, &validation),
        generic: vec![leaf(2, &validation), leaf(3, &validation)],
        last: Some(Box::new(leaf(4, &validation))),
    };
    value.validate().unwrap();
    assert_eq!(*validation.borrow(), [1, 2, 3, 4]);
    validation.borrow_mut().clear();
    value.generic[0].invalid = true;
    assert!(value.validate().is_err());
    assert_eq!(*validation.borrow(), [1, 2]);
    let mut visit = visitor();
    visit.fail_at = Some(2);
    assert_eq!(value.visit_arrays(&mut visit), Err(2));
    assert_eq!(visit.seen, [(1, u16::MAX), (2, u16::MAX)]);
    assert!(matches!(&value.first.values, Array::Inline(values) if values.is_empty()));
    assert!(matches!(&value.generic[0].values, Array::Inline(values) if values.len() == 1));
    let mut tuple = Tuple(leaf(5, &validation), [leaf(6, &validation)]);
    let mut visit = visitor();
    tuple.visit_arrays(&mut visit).unwrap();
    assert_eq!(visit.seen, [(5, u16::MAX), (6, u16::MAX)]);
    Unit.validate().unwrap();
    Unit.visit_arrays(&mut visit).unwrap();
    assert_eq!(visit.seen.len(), 2);
}
#[test]
fn enum_shapes_dispatch_only_the_selected_variant() {
    let validation = Rc::new(RefCell::new(Vec::new()));
    let mut values = [
        Choice::Named {
            first: leaf(7, &validation),
            second: leaf(8, &validation),
        },
        Choice::Tuple(leaf(9, &validation), leaf(10, &validation)),
        Choice::Unit,
    ];
    let mut visit = visitor();
    for value in &mut values {
        value.validate().unwrap();
        value.visit_arrays(&mut visit).unwrap();
    }
    assert_eq!(*validation.borrow(), [7, 8, 9, 10]);
    assert_eq!(
        visit.seen,
        [(7, u16::MAX), (8, u16::MAX), (9, u16::MAX), (10, u16::MAX)]
    );
}
#[derive(m2::Validate, m2::VisitArrays)]
struct Animated<'data> {
    track: Track<'data, f32, V264>,
}
#[test]
fn native_tracks_keep_sequence_and_global_context() {
    let mut animated = Animated {
        track: Track {
            base: TrackBase {
                interpolation: Interpolation::Linear,
                global_sequence: 12,
                timeline: ModernTimeline {
                    timestamps: vec![Array::Inline(ZeroVec::alloc_from_slice(&[0, 20]))],
                },
            },
            values: TrackValues::Values(vec![Array::Inline(ZeroVec::alloc_from_slice(&[
                1.0, 2.0,
            ]))]),
        },
    };
    animated.validate().unwrap();
    let mut visit = visitor();
    animated.visit_arrays(&mut visit).unwrap();
    assert_eq!(visit.seen, [(0, 12), (0, 12)]);
    animated.validate().unwrap();
    animated.track.base.global_sequence = u16::MAX;
    animated
        .track
        .base
        .timeline
        .timestamps
        .push(Array::Inline(ZeroVec::alloc_from_slice(&[50])));
    let TrackValues::Values(values) = &mut animated.track.values else {
        unreachable!()
    };
    values.push(Array::Inline(ZeroVec::alloc_from_slice(&[3.0])));
    animated.validate().unwrap();
    let mut visit = visitor();
    animated.visit_arrays(&mut visit).unwrap();
    assert_eq!(
        visit.seen,
        [(0, u16::MAX), (1, u16::MAX), (0, u16::MAX), (1, u16::MAX)]
    );
}

#[derive(m2::Validate, m2::VisitArrays)]
struct ResultParameter<Result> {
    value: Result,
}
#[derive(m2::Validate, m2::VisitArrays)]
struct VisitorParameter<__M2Visitor> {
    value: __M2Visitor,
}
#[allow(non_upper_case_globals)]
mod const_names {
    use super::*;

    #[derive(m2::Validate, m2::VisitArrays)]
    pub(super) struct VisitorConst<const visitor: usize> {
        pub(super) value: [u32; visitor],
    }
    #[derive(m2::Validate, m2::VisitArrays)]
    pub(super) enum FieldConst<const __m2_field_0: usize> {
        Value { value: [u32; __m2_field_0] },
    }
}
use const_names::{FieldConst, VisitorConst};
#[derive(m2::Validate, m2::VisitArrays)]
enum Empty {}
#[derive(m2::Read, m2::Write, m2::RecordLayout)]
struct CodecResult<Result> {
    value: Result,
}
#[test]
fn generated_names_do_not_shadow_user_parameters() {
    let mut visit = visitor();
    let mut result = ResultParameter { value: 7u32 };
    result.validate().unwrap();
    result.visit_arrays(&mut visit).unwrap();
    let mut named = VisitorParameter { value: 8u32 };
    named.validate().unwrap();
    named.visit_arrays(&mut visit).unwrap();
    let mut constant = VisitorConst::<2> { value: [1, 2] };
    constant.validate().unwrap();
    constant.visit_arrays(&mut visit).unwrap();
    let mut field = FieldConst::<2>::Value { value: [3, 4] };
    field.validate().unwrap();
    field.visit_arrays(&mut visit).unwrap();
    assert!(visit.seen.is_empty());
    fn assert_empty<T: m2::Validate + for<'data> m2::VisitArrays<'data>>() {}
    assert_empty::<Empty>();
    let codec = CodecResult { value: 0x1234u32 };
    let mut encoder = m2::codec::Encoder::new(264, ModelFlags::default());
    m2::Write::write(&codec, &mut encoder).unwrap();
    let bytes = encoder.finish();
    let mut decoder = m2::codec::Decoder::new(&bytes, 264, ModelFlags::default());
    let decoded = <CodecResult<u32> as m2::Read>::read(&mut decoder).unwrap();
    assert_eq!(decoded.value, codec.value);
}
