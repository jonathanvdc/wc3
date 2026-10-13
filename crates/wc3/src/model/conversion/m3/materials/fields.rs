//! Fields mappings for native M3 materials.
use super::{native, nr, wem, wr, Error};
pub(in crate::model::conversion::m3) trait Slot<T> {
    fn value(&self) -> T;
}
impl<T: Default> Slot<T> for () {
    fn value(&self) -> T {
        T::default()
    }
}
macro_rules! slot {($($t:ty),*)=>{$(impl Slot<$t> for $t{fn value(&self)->Self{*self}})*};}
slot!(f32, u32, [f32; 2], [f32; 3]);
impl<T: native::Target> Slot<native::Ref<T>> for native::Ref<T> {
    fn value(&self) -> Self {
        *self
    }
}
impl<T: Clone> Slot<nr::AnimationValue<T>> for nr::AnimationValue<T> {
    fn value(&self) -> Self {
        self.clone()
    }
}

pub(in crate::model::conversion::m3) fn anim<T>(r: nr::AnimationValue<T>) -> wr::M3AnimRef<T> {
    wr::M3AnimRef {
        interp_type: r.interpolation,
        flags: r.flags,
        anim_id: r.animation_id,
        init_value: r.initial_value,
        null_value: r.null_value,
        unused: r.reserved,
    }
}

pub(in crate::model::conversion::m3) fn color(
    r: nr::AnimationValue<[u8; 4]>,
) -> wr::M3AnimRef<wr::M3ColorBGRA> {
    let c = |r: [u8; 4]| wr::M3ColorBGRA {
        b: r[0],
        g: r[1],
        r: r[2],
        a: r[3],
    };
    wr::M3AnimRef {
        interp_type: r.interpolation,
        flags: r.flags,
        anim_id: r.animation_id,
        init_value: c(r.initial_value),
        null_value: c(r.null_value),
        unused: r.reserved,
    }
}

pub(in crate::model::conversion::m3) fn put<T: wem::Record + wem::Write>(
    doc: &mut wem::RawDocument<'static>,
    rows: &[T],
) -> Result<wem::Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(wem::Ref::default());
    }
    let id = doc.append(rows)?;
    Ok(wem::Ref::from_section(rows.len() as u32, id.index()))
}

pub(in crate::model::conversion::m3) fn text(
    doc: &mut wem::RawDocument<'static>,
    value: &str,
) -> Result<wem::Ref<wr::targets::Character>, Error> {
    Ok(put(
        doc,
        &value
            .bytes()
            .map(|value| wr::Character { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}

pub(in crate::model::conversion::m3) fn native_anim<T>(
    r: wr::M3AnimRef<T>,
) -> nr::AnimationValue<T> {
    nr::AnimationValue {
        interpolation: r.interp_type,
        flags: r.flags,
        animation_id: r.anim_id,
        initial_value: r.init_value,
        null_value: r.null_value,
        reserved: r.unused,
    }
}

pub(in crate::model::conversion::m3) fn native_color(
    r: wr::M3AnimRef<wr::M3ColorBGRA>,
) -> nr::AnimationValue<[u8; 4]> {
    let c = |r: wr::M3ColorBGRA| [r.b, r.g, r.r, r.a];
    nr::AnimationValue {
        interpolation: r.interp_type,
        flags: r.flags,
        animation_id: r.anim_id,
        initial_value: c(r.init_value),
        null_value: c(r.null_value),
        reserved: r.unused,
    }
}
