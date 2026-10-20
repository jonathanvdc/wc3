//! Checked immutable event-to-root field maps; target layouts may be supersets.
use crate::model::scene::popcorn::ValueType;
use std::collections::HashSet;
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub name: Box<[u8]>,
    pub native_type: u32,
    pub kind: Option<ValueType>,
    pub footprint: u32,
    pub role: u32,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Layout {
    pub fields: Box<[Field]>,
    pub position: Option<usize>,
    pub orientation: Option<usize>,
}
impl Layout {
    pub fn new(fields: Box<[Field]>) -> Result<Self> {
        let mut names = HashSet::new();
        let mut position = None;
        let mut orientation = None;
        for (index, field) in fields.iter().enumerate() {
            if !names.insert(field.name.as_ref()) {
                return Err("duplicate payload declaration name");
            }
            if let Some(kind) = field.kind {
                if field.footprint as usize != kind.components() * 4 {
                    return Err("payload declaration footprint disagrees with typed layout");
                }
            }
            match field.role {
                0 => {}
                1 if field.kind == Some(ValueType::Float3) => {
                    if position.replace(index).is_some() {
                        return Err("payload layout declares multiple position channels");
                    }
                }
                2 if field.kind == Some(ValueType::Quaternion) => {
                    if orientation.replace(index).is_some() {
                        return Err("payload layout declares multiple orientation channels");
                    }
                }
                3 if field.kind == Some(ValueType::Int) => {}
                _ => return Err("unsupported payload declaration role or typed layout"),
            }
        }
        Ok(Self {
            fields,
            position,
            orientation,
        })
    }
    pub fn map_to(&self, target: &Self) -> Result<Box<[usize]>> {
        self.fields
            .iter()
            .map(|field| {
                let (index, other) = target
                    .fields
                    .iter()
                    .enumerate()
                    .find(|(_, other)| other.name == field.name)
                    .ok_or("outgoing payload field is absent from target root layout")?;
                if field.native_type != other.native_type
                    || field.kind != other.kind
                    || field.footprint != other.footprint
                    || field.role != other.role
                {
                    return Err("outgoing payload declaration conflicts with target root layout");
                }
                Ok(index)
            })
            .collect()
    }
    pub fn validate_reads(&self, mapping: &[usize], reads: &[usize]) -> Result<()> {
        for read in reads {
            if *read >= self.fields.len() || !mapping.contains(read) {
                return Err(
                    "incoming route does not supply a root payload channel read by the programme",
                );
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn field(name: &[u8], kind: ValueType, native_type: u32, role: u32) -> Field {
        Field {
            name: name.into(),
            footprint: kind.components() as u32 * 4,
            kind: Some(kind),
            native_type,
            role,
        }
    }
    #[test]
    fn outgoing_layout_maps_exact_names_into_sparse_target_superset() {
        let outgoing =
            Layout::new(vec![field(b"Position", ValueType::Float3, 33, 1)].into()).unwrap();
        let target = Layout::new(
            vec![
                field(b"Size", ValueType::Float, 31, 0),
                field(b"Position", ValueType::Float3, 33, 1),
            ]
            .into(),
        )
        .unwrap();
        assert_eq!(&*outgoing.map_to(&target).unwrap(), &[1]);
        assert_eq!(target.position, Some(1));
        assert!(target.validate_reads(&[1], &[1]).is_ok());
        assert!(target.validate_reads(&[1], &[0]).is_err());
        let conflict =
            Layout::new(vec![field(b"Position", ValueType::Float4, 34, 0)].into()).unwrap();
        assert!(outgoing.map_to(&conflict).is_err());
    }
    #[test]
    fn layout_rejects_ambiguous_roles_bad_footprints_and_duplicate_names() {
        assert!(Layout::new(
            vec![
                field(b"A", ValueType::Float3, 33, 1),
                field(b"B", ValueType::Float3, 33, 1)
            ]
            .into()
        )
        .is_err());
        assert!(Layout::new(
            vec![
                field(b"A", ValueType::Float, 31, 0),
                field(b"A", ValueType::Float, 31, 0)
            ]
            .into()
        )
        .is_err());
        let mut malformed = field(b"A", ValueType::Float3, 33, 1);
        malformed.footprint = 4;
        assert!(Layout::new(vec![malformed].into()).is_err());
    }
}
