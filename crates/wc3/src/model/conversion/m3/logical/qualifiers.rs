//! Uninterpreted physical reference controls must never silently disappear.
use super::materials::read_error;
use crate::model::conversion::{Context, Error};
use crate::model::m3::{
    records::{known_record_size, visit_known_references},
    Document, Envelope, RawReference, ReadError, ReferenceVisitor,
};
use std::collections::{BTreeMap, VecDeque};
#[derive(Default)]
struct References(Vec<RawReference>);
impl ReferenceVisitor for References {
    type Error = ReadError;
    fn visit(&mut self, reference: RawReference) -> Result<(), ReadError> {
        self.0.push(reference);
        Ok(())
    }
}
pub(super) fn check<E: Envelope>(
    document: &Document<'_, E>,
    context: &mut Context,
) -> Result<(), Error> {
    let root = document.root_reference();
    if root.flags.bits() != 0 {
        context.unsupported("m3.reference_qualifier.root", "nonzero root reference qualifiers have no established meaning or logical encoding hint mapping")?;
    }
    let mut queue = VecDeque::from([root]);
    let mut visited = BTreeMap::<u32, u32>::new();
    while let Some(reference) = queue.pop_front() {
        if reference.entries == 0
            || visited
                .get(&reference.index)
                .is_some_and(|count| *count >= reference.entries)
        {
            continue;
        }
        visited.insert(reference.index, reference.entries);
        let section = document
            .section_id(reference.index)
            .and_then(|id| document.section(id))
            .ok_or_else(|| Error::new("m3.reference_qualifier", "reference target is absent"))?;
        let mut references = References::default();
        let Some(width) = known_record_size::<E>(section.tag(), section.revision()) else {
            continue;
        };
        let length = width
            .checked_mul(reference.entries as usize)
            .ok_or_else(|| {
                Error::new(
                    "m3.reference_qualifier",
                    "reference span overflows address capacity",
                )
            })?;
        let bytes = section.bytes().get(..length).ok_or_else(|| {
            Error::new(
                "m3.reference_qualifier",
                "reference span exceeds target buffer",
            )
        })?;
        visit_known_references::<E, _>(
            section.tag(),
            section.revision(),
            bytes,
            reference.entries,
            &mut references,
        )
        .map_err(read_error)?;
        for (field, reference) in references.0.into_iter().enumerate() {
            if reference.flags.bits() != 0 {
                context.unsupported(format!("m3.reference_qualifier.{}.field[{field}]",String::from_utf8_lossy(&section.tag())), "nonzero native reference qualifiers have no established meaning or logical encoding hint mapping")?;
            }
            queue.push_back(reference);
        }
    }
    Ok(())
}
