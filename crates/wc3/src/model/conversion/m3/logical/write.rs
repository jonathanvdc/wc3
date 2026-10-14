//! Shared mechanical output helpers relocate new references and preserve property controls.
use super::animation::AnimationIds;
use crate::model::conversion::{Context, Error};
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::profiles::m3::Property;
use native::{Envelope, Ref};
pub(super) fn append<
    E: Envelope,
    T: native::Record + native::Write<E> + native::RecordLayout<E>,
>(
    document: &mut native::Document<'_, E>,
    rows: &[T],
) -> Result<Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(rows.len())
        .map_err(|_| Error::new("m3.array", "native count capacity exceeded"))?;
    let id = document
        .append(rows)
        .map_err(|e| Error::new("m3.write", e.to_string()))?;
    Ok(Ref::from_section(count, id.index()))
}
pub(super) fn animated<T: Clone>(
    property: &Property<T>,
    ids: &AnimationIds,
) -> nr::AnimationValue<T> {
    let mapped = property
        .binding
        .as_ref()
        .and_then(|binding| ids.get(binding.target()));
    nr::AnimationValue {
        interpolation: nr::Interpolation::from_raw(
            mapped
                .map_or(property.control.interpolation, |x| x.interpolation)
                .raw(),
        ),
        flags: nr::AnimationFlags(
            property.control.flags.bits()
                | if mapped.is_some_and(|x| x.keyed) {
                    6
                } else {
                    0
                },
        ),
        animation_id: mapped.map_or(property.control.source_id, |x| x.id),
        initial_value: property.initial.clone(),
        null_value: property.reset.clone(),
        reserved: property.control.reserved,
    }
}
pub(super) fn string<E: Envelope>(
    document: &mut native::Document<'_, E>,
    text: &str,
) -> Result<Ref<nr::targets::Character>, Error> {
    if text.as_bytes().contains(&0) {
        return Err(Error::new(
            "m3.string",
            "native text cannot contain embedded NUL",
        ));
    }
    let mut characters: Vec<_> = text.bytes().map(|value| nr::Character { value }).collect();
    characters.push(nr::Character { value: 0 });
    Ok(append(document, &characters)?.cast())
}

/// Quantize the source color leaf while retaining its independent animation control.
pub(super) fn color(
    property: &Property<[f32; 4]>,
    ids: &AnimationIds,
    context: &mut Context,
) -> Result<nr::AnimationValue<[u8; 4]>, Error> {
    fn quantize(value: [f32; 4], context: &mut Context) -> Result<[u8; 4], Error> {
        let mut result = [0; 4];
        for (index, value) in value.into_iter().enumerate() {
            if !(0.0..=1.0).contains(&value) {
                return Err(Error::new(
                    "m3.color",
                    "color component exceeds normalized byte range",
                ));
            }
            let byte = (value * 255.0).round() as u8;
            context.approximate(
                "m3.color",
                (f32::from(byte) / 255.0 - value).abs(),
                "color requires native byte quantization",
            )?;
            result[index] = byte;
        }
        Ok([result[2], result[1], result[0], result[3]])
    }
    let value = animated(property, ids);
    Ok(nr::AnimationValue {
        interpolation: value.interpolation,
        flags: value.flags,
        animation_id: value.animation_id,
        initial_value: quantize(value.initial_value, context)?,
        null_value: quantize(value.null_value, context)?,
        reserved: value.reserved,
    })
}
pub(super) fn fixed_color(
    property: &Property<[f32; 4]>,
    ids: &AnimationIds,
    context: &mut Context,
) -> Result<nr::AnimationValue<nr::FixedBytes<4>>, Error> {
    let value = color(property, ids, context)?;
    Ok(nr::AnimationValue {
        interpolation: value.interpolation,
        flags: value.flags,
        animation_id: value.animation_id,
        initial_value: nr::FixedBytes(value.initial_value),
        null_value: nr::FixedBytes(value.null_value),
        reserved: value.reserved,
    })
}
