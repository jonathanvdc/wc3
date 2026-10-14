//! Borrowed attribute views over native packed vertex fields.
use crate::model::conversion::Error;

use crate::model::m3::{self as native};
use crate::model::semantic::Values;
use crate::visit_m3_vertices;
use native::Envelope;
use std::array::from_fn;

#[derive(Clone)]
pub(super) struct Packed<'a> {
    source: &'a [u8],
    stride: usize,
    count: usize,
    color: bool,
    uv: [Option<usize>; 5],
}
impl<'a> Packed<'a> {
    pub fn read<E: Envelope>(model: &'a native::Model<'_, E>) -> Result<Self, Error> {
        let root = model
            .root_record()
            .map_err(|e| Error::new("m3.root", e.to_string()))?;
        let source = model
            .vertices()
            .map_err(|e| Error::new("m3.vertices", e.to_string()))?;
        let count = if source.as_bytes().is_empty() {
            0
        } else {
            source
                .vertex_count()
                .ok_or_else(|| Error::new("m3.vertices", "packed attribute layout is unresolved"))?
        };
        let stride = source.as_bytes().len().checked_div(count).unwrap_or(0);
        let color = root.vertex_flags().color();
        let flags = root.vertex_flags().bits();
        let mask = ((flags >> 17) & 15) | (((flags >> 30) & 1) << 4);
        let mut offset = 24 + usize::from(color) * 4;
        let uv = from_fn(|channel| {
            if mask & (1 << channel) == 0 {
                None
            } else {
                let at = offset;
                offset += 4;
                Some(at)
            }
        });
        Ok(Self {
            source: visit_m3_vertices!(source, |vertices| vertices.into_storage()),
            stride,
            count,
            color,
            uv,
        })
    }
    pub fn len(&self) -> usize {
        self.count
    }
    fn row(&self, index: usize) -> &[u8] {
        &self.source[index * self.stride..(index + 1) * self.stride]
    }
    fn view<T: Clone + Send + Sync + 'a>(
        &self,
        start: usize,
        count: usize,
        map: impl Fn(&[u8]) -> T + Send + Sync + 'a,
    ) -> Values<'a, T> {
        let packed = self.clone();
        Values::view(count, move |index| map(packed.row(start + index)))
    }
    pub fn positions(&self, start: usize, count: usize) -> Values<'a, [f32; 3]> {
        self.view(start, count, |row| {
            [0, 4, 8].map(|offset| f32::from_le_bytes(row[offset..offset + 4].try_into().unwrap()))
        })
    }
    pub fn normals(&self, start: usize, count: usize) -> Values<'a, [f32; 3]> {
        self.view(start, count, |row| {
            [20, 21, 22].map(|offset| f32::from(row[offset]) / 255. * 2. - 1.)
        })
    }
    pub fn tangents(&self, start: usize, count: usize) -> Values<'a, [f32; 4]> {
        if count == 0 {
            return Values::default();
        }
        let at = self.stride - 4;
        self.view(start, count, move |row| {
            [at, at + 1, at + 2, at + 3].map(|offset| f32::from(row[offset]) / 255. * 2. - 1.)
        })
    }
    pub fn colors(&self, start: usize, count: usize) -> Values<'a, [f32; 4]> {
        if !self.color {
            return Values::default();
        }
        self.view(start, count, |row| {
            [26, 25, 24, 27].map(|offset| f32::from(row[offset]) / 255.)
        })
    }
    pub fn uv_sets(
        &self,
        start: usize,
        count: usize,
        scale: f32,
        offset: f32,
    ) -> Vec<Values<'a, [f32; 2]>> {
        let Some(last) = self.uv.iter().rposition(Option::is_some) else {
            return Vec::new();
        };
        self.uv[..=last]
            .iter()
            .map(|at| {
                let Some(at) = *at else {
                    return Values::default();
                };
                self.view(start, count, move |row| {
                    [at, at + 2].map(|lane| {
                        f32::from(i16::from_le_bytes(row[lane..lane + 2].try_into().unwrap()))
                            * scale
                            + offset
                    })
                })
            })
            .collect()
    }

    pub fn skin(&self, index: usize) -> ([u8; 4], [u8; 4]) {
        let row = self.row(index);
        (
            row[12..16].try_into().unwrap(),
            row[16..20].try_into().unwrap(),
        )
    }
    pub fn influences<T: Clone + Send + Sync + 'a>(
        &self,
        start: usize,
        count: usize,
        map: impl Fn([u8; 4], [u8; 4]) -> T + Send + Sync + 'a,
    ) -> Values<'a, T> {
        let packed = self.clone();
        Values::view(count, move |i| {
            let (weights, joints) = packed.skin(start + i);
            map(weights, joints)
        })
    }
}
