//! Named scalar metadata for source controls without a shared WEM field.
use super::{nr, wem, wr, Error};
use std::collections::BTreeMap;
pub(super) fn put<T: wem::Record + wem::Write>(
    doc: &mut wem::RawDocument<'static>,
    rows: &[T],
) -> Result<wem::Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(wem::Ref::default());
    }
    let id = doc.append(rows)?;
    Ok(wem::Ref::from_section(rows.len() as u32, id.index()))
}
pub(super) fn text(
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
pub(in crate::model::conversion::m3) fn read(
    doc: &wem::RawDocument<'_>,
    bag: &wr::NativeBag,
) -> Result<BTreeMap<String, i64>, Error> {
    let mut values = BTreeMap::new();
    if bag.entries.entries == 0 {
        return Ok(values);
    }
    for row in doc
        .resolve::<wr::NativeEntry<wr::V2>>(bag.entries.cast())?
        .iter()
    {
        let row = row?;
        values.insert(
            doc.string(row.name)?.unwrap_or_default().to_owned(),
            row.value,
        );
    }
    Ok(values)
}
pub(in crate::model::conversion::m3) fn animated<T: Components>(
    values: &mut BTreeMap<String, i64>,
    key: &str,
    value: nr::AnimationValue<T>,
) {
    values.insert(
        format!("m3.{key}.interpolation"),
        i64::from((value.interpolation).raw()),
    );
    values.insert(format!("m3.{key}.flags"), i64::from((value.flags).bits()));
    values.insert(
        format!("m3.{key}.animation_id"),
        i64::from(value.animation_id),
    );
    values.insert(format!("m3.{key}.reserved"), i64::from(value.reserved));
    for (index, value) in value.null_value.components().into_iter().enumerate() {
        values.insert(
            format!("m3.{key}.null[{index}]"),
            i64::from(value.to_bits()),
        );
    }
}
pub(in crate::model::conversion::m3) fn restore<T: Components>(
    values: &BTreeMap<String, i64>,
    key: &str,
    initial: T,
) -> nr::AnimationValue<T> {
    let get = |name: &str| {
        values
            .get(&format!("m3.{key}.{name}"))
            .copied()
            .unwrap_or(0)
    };
    let null = (0..initial.components().len())
        .map(|i| {
            values
                .get(&format!("m3.{key}.null[{i}]"))
                .map(|&v| f32::from_bits(v as u32))
                .unwrap_or(initial.components()[i])
        })
        .collect::<Vec<_>>();
    nr::AnimationValue {
        interpolation: nr::Interpolation::from_raw(get("interpolation") as u16),
        flags: nr::AnimationFlags(get("flags") as u16),
        animation_id: get("animation_id") as u32,
        reserved: get("reserved") as i32,
        initial_value: initial,
        null_value: T::from_components(&null),
    }
}
pub(in crate::model::conversion::m3) trait Components:
    Copy
{
    fn components(self) -> Vec<f32>;
    fn from_components(values: &[f32]) -> Self;
}
impl Components for f32 {
    fn components(self) -> Vec<f32> {
        vec![self]
    }
    fn from_components(values: &[f32]) -> Self {
        values[0]
    }
}
impl<const N: usize> Components for [f32; N] {
    fn components(self) -> Vec<f32> {
        self.to_vec()
    }
    fn from_components(values: &[f32]) -> Self {
        values.try_into().unwrap()
    }
}

pub(in crate::model::conversion::m3) fn extend(
    doc: &mut wem::RawDocument<'static>,
    source: &wr::NativeBag,
    values: &BTreeMap<String, i64>,
) -> Result<wr::NativeBag, Error> {
    let mut rows = doc
        .resolve::<wr::NativeEntry<wr::V2>>(source.entries.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    for (key, value) in values {
        let index = rows
            .iter()
            .position(|row| doc.string(row.name).ok().flatten() == Some(key.as_str()));
        if let Some(index) = index {
            rows[index].value = *value;
        } else {
            rows.push(wr::NativeEntry::<wr::V2> {
                name: text(doc, key)?,
                value: *value,
                text: wem::Ref::default(),
            });
        }
    }
    Ok(wr::NativeBag {
        entries: put(doc, &rows)?.cast(),
    })
}
pub(in crate::model::conversion::m3) fn set_text(
    doc: &mut wem::RawDocument<'static>,
    source: &wr::NativeBag,
    key: &str,
    value: &str,
) -> Result<wr::NativeBag, Error> {
    let mut rows = doc
        .resolve::<wr::NativeEntry<wr::V2>>(source.entries.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let reference = text(doc, value)?;
    if let Some(row) = rows
        .iter_mut()
        .find(|row| doc.string(row.name).ok().flatten() == Some(key))
    {
        row.text = reference;
    } else {
        rows.push(wr::NativeEntry::<wr::V2> {
            name: text(doc, key)?,
            text: reference,
            value: 0,
        });
    }
    Ok(wr::NativeBag {
        entries: put(doc, &rows)?.cast(),
    })
}
pub(super) fn get_text<'view>(
    doc: &'view wem::RawDocument<'_>,
    source: &wr::NativeBag,
    key: &str,
) -> Result<Option<&'view str>, Error> {
    for row in doc
        .resolve::<wr::NativeEntry<wr::V2>>(source.entries.cast())?
        .iter()
    {
        let row = row?;
        if doc.string(row.name)? == Some(key) {
            return Ok(doc.string(row.text)?);
        }
    }
    Ok(None)
}

impl Components for u32 {
    fn components(self) -> Vec<f32> {
        vec![f32::from_bits(self)]
    }
    fn from_components(values: &[f32]) -> Self {
        values[0].to_bits()
    }
}
pub(in crate::model::conversion::m3) fn header<T>(
    values: &mut BTreeMap<String, i64>,
    key: &str,
    value: &nr::AnimationValue<T>,
) {
    values.insert(
        format!("m3.{key}.interpolation"),
        i64::from((value.interpolation).raw()),
    );
    values.insert(format!("m3.{key}.flags"), i64::from((value.flags).bits()));
    values.insert(
        format!("m3.{key}.animation_id"),
        i64::from(value.animation_id),
    );
    values.insert(format!("m3.{key}.reserved"), i64::from(value.reserved));
}
pub(super) fn property<T: Copy>(
    values: &mut BTreeMap<String, i64>,
    key: &str,
    value: nr::AnimationValue<T>,
) -> wr::Sc2Property<T> {
    header(values, key, &value);
    wr::Sc2Property {
        init_value: value.initial_value,
        null_value: value.null_value,
    }
}
pub(in crate::model::conversion::m3) fn native_property<T>(
    values: &BTreeMap<String, i64>,
    key: &str,
    value: wr::Sc2Property<T>,
) -> nr::AnimationValue<T> {
    let get = |field| {
        values
            .get(&format!("m3.{key}.{field}"))
            .copied()
            .unwrap_or(0)
    };
    nr::AnimationValue {
        interpolation: nr::Interpolation::from_raw(get("interpolation") as u16),
        flags: nr::AnimationFlags(get("flags") as u16),
        animation_id: get("animation_id") as u32,
        reserved: get("reserved") as i32,
        initial_value: value.init_value,
        null_value: value.null_value,
    }
}
