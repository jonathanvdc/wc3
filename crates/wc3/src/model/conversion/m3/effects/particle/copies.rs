//! Explicit shared emitter instances and their two editable overrides.
use super::super::emitter_fields::native_squirt;
use super::{native, native_property, nr, wr, BTreeMap, Context, Envelope, Error, Ref};
type EncodedCopies = (
    Vec<Row>,
    Vec<Ref<nr::targets::U32>>,
    Ref<nr::targets::ParticleEmitterCopy>,
);
type Row = (wr::Sc2ParticleEmitterPayload, BTreeMap<String, i64>, u32);
pub(super) fn encode<E: Envelope>(
    input: &[Row],
    doc: &mut native::Document<'static, E>,
    context: &mut Context,
) -> Result<EncodedCopies, Error> {
    let mut rows: Vec<Row> = input
        .iter()
        .filter(|r| r.0.copy_of == u32::MAX)
        .cloned()
        .collect();
    let mut indices = vec![Vec::<nr::U32>::new(); rows.len()];
    let mut copies = Vec::new();
    for (payload, values, bone) in input.iter().filter(|r| r.0.copy_of != u32::MAX) {
        let parent = rows
            .iter()
            .position(|r| {
                r.1.get("m3.effect.wem_index").copied() == Some(i64::from(payload.copy_of))
            })
            .ok_or_else(|| {
                Error::new(
                    "particle.copy_of",
                    "copy parent is not an independent emitter",
                )
            })?;
        let mut comparable = payload.clone();
        comparable.copy_of = u32::MAX;
        comparable.emission_rate = rows[parent].0.emission_rate.clone();
        comparable.squirt_amount = rows[parent].0.squirt_amount.clone();
        if comparable != rows[parent].0 {
            return Err(Error::new(
                "particle.copy_of",
                "copy edits outside emission rate and squirt amount require an independent emitter",
            ));
        }
        let index = copies.len() as u32;
        copies.push(nr::ParticleEmitterCopy {
            bone_index: *bone,
            emission_rate: native_property(
                values,
                "particle.emission_rate",
                payload.emission_rate.clone(),
            ),
            squirt_amount: native_squirt(
                values,
                "particle.squirt_amount",
                payload.squirt_amount.clone(),
                context,
            )?,
        });
        indices[parent].push(nr::U32 { value: index });
    }
    let mut refs = Vec::new();
    for values in indices {
        refs.push(super::super::super::append(doc, &values)?.cast());
    }
    Ok((
        std::mem::take(&mut rows),
        refs,
        super::super::super::append(doc, &copies)?.cast(),
    ))
}
