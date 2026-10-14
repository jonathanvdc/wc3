//! Animation split declarations retain ordered typed controls without guessing a bone palette.
use super::{
    materials::{read_error, string},
    write::{append, string as write_string},
    Graph, GraphBuilder,
};
use crate::model::conversion::Error;
use crate::model::m3::{records, Document, Envelope, Ref};
use crate::model::semantic::{
    profiles::m3::{values::BoneSetFlags, AnimationPartition, System},
    Values,
};
pub(super) fn read<'a, E: Envelope + Send + Sync>(
    document: &'a Document<'_, E>,
    reference: Ref<records::targets::BoneAnimationSet>,
    builder: &mut GraphBuilder<'a>,
) -> Result<(), Error> {
    let mut declarations = Vec::new();
    for row in document
        .resolve::<records::BoneAnimationSet>(reference.cast())
        .map_err(read_error)?
        .iter()
    {
        let row = row.map_err(read_error)?;
        let items = document
            .resolve::<records::U16>(row.split_items.cast())
            .map_err(read_error)?;
        for item in items.iter() {
            item.map_err(read_error)?;
        }
        declarations.push(AnimationPartition {
            name: string(document, row.name)?,
            flags: BoneSetFlags(row.flags.bits()),
            selector: row.animation_sequence_index,
            fallback_selector: row.fallback_sequence_index,
            split_items: Values::view(items.len(), move |index| {
                items
                    .get(index)
                    .expect("checked split item")
                    .expect("validated split item")
                    .value
            }),
        });
    }
    if !declarations.is_empty() {
        builder.add_system(System::AnimationPartitions(declarations))?;
    }
    Ok(())
}
pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Document<'_, E>,
) -> Result<Ref<records::targets::BoneAnimationSet>, Error> {
    let mut rows = Vec::new();
    for (_, system) in graph.systems() {
        let System::AnimationPartitions(declarations) = system else {
            continue;
        };
        for declaration in declarations {
            let items: Vec<_> = declaration
                .split_items
                .iter()
                .map(|value| records::U16 { value })
                .collect();
            rows.push(records::BoneAnimationSet {
                name: write_string(document, &declaration.name)?,
                flags: records::BoneSetFlags(declaration.flags.bits()),
                animation_sequence_index: declaration.selector,
                fallback_sequence_index: declaration.fallback_selector,
                split_items: append(document, &items)?.cast(),
            });
        }
    }
    Ok(append(document, &rows)?.cast())
}
