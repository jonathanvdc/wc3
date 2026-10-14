//! Mechanical composite surface encoding.
use super::{append, native_anim, nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::Composite,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::CompositeMaterial<nr::V2>, Error> {
    Ok(nr::CompositeMaterial {
        name: writer.string(name)?,
        priority: r.priority,
        sections: append(
            writer.document,
            &r.sections
                .iter()
                .map(|r| {
                    Ok(nr::CompositeSection {
                        material_index: writer.material(r.material)?,
                        map_multiplier: native_anim(&r.map_multiplier, writer.animation_ids),
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?,
        )?
        .cast(),
    })
}
