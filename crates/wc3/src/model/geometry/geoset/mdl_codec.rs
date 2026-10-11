//! Derived geoset fields with adapters for the mesh-specific list framing.
use super::{
    Geoset, GeosetExtent, GeosetExtraSection, GeosetExtraSections, GeosetLevelOfDetail, SkinWeight,
};
use crate::model::IoError;
use crate::model::ZeroVec;
use crate::model::{mdl, FixedText, ModelVersion, Vec3};
use mdl::WriteProperty as _;
use mdl::{Dialect, Field, Parser, Span, TokenKind, Writer};
use std::io::Write as IoWrite;
use zerovec::ule::AsULE;

#[derive(mdl::Read, mdl::Write)]
#[mdl(entry)]
struct Entry<T>(T);

pub(super) struct List<T: AsULE>(ZeroVec<'static, T>);
pub(super) struct Uncounted<T: AsULE>(ZeroVec<'static, T>);
pub(super) struct OptionalList<T: AsULE>(Option<ZeroVec<'static, T>>);

fn read_list<T: mdl::Read + AsULE>(
    parser: &mut Parser<'_>,
) -> Result<ZeroVec<'static, T>, mdl::ReadError> {
    parser
        .counted::<Entry<T>>()?
        .map(|item| item.map(|entry| entry.0))
        .collect()
}
fn read_values<T: mdl::Read + AsULE>(
    parser: &mut Parser<'_>,
) -> Result<ZeroVec<'static, T>, mdl::ReadError> {
    parser.expect(TokenKind::OpenBrace)?;
    let mut values = Vec::new();
    while !parser.consume(TokenKind::CloseBrace)? {
        values.push(parser.read_property::<T>()?.to_unaligned());
    }
    Ok(ZeroVec::new_owned(values))
}
impl<T: mdl::Read + AsULE> mdl::ReadProperty for List<T> {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        read_list(parser).map(Self)
    }
}
impl<T: mdl::Read + AsULE> mdl::ReadProperty for Uncounted<T> {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        read_values(parser).map(Self)
    }
    fn missing_mdl_property(_: &'static str, _: Span) -> Result<Self, mdl::ReadError> {
        // Reforged skinning can leave the legacy VertexGroup block absent.
        // Geoset validation still checks that complete skin rows replace it.
        Ok(Self(ZeroVec::new()))
    }
}
impl<T: mdl::Read + AsULE> mdl::ReadProperty for OptionalList<T> {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        read_list(parser).map(|values| Self(Some(values)))
    }
    fn missing_mdl_property(_: &'static str, _: Span) -> Result<Self, mdl::ReadError> {
        Ok(Self(None))
    }
}
pub(super) struct ListRef<'a, T: AsULE>(pub(super) &'a ZeroVec<'a, T>);
pub(super) struct UncountedRef<'a, T: AsULE>(&'a ZeroVec<'a, T>);
pub(super) struct OptionalListRef<'a, T: AsULE>(Option<&'a ZeroVec<'a, T>>);
impl<T: AsULE + mdl::Write> mdl::WriteProperty for ListRef<'_, T> {
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        writer.begin_counted_block(name, self.0.len())?;
        for value in self.0.iter() {
            writer.entry(&value)?;
        }
        writer.end_block()
    }
}
impl<T: AsULE + mdl::Write> mdl::WriteProperty for UncountedRef<'_, T> {
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        writer.begin_block(name)?;
        for value in self.0.iter() {
            writer.entry(&value)?;
        }
        writer.end_block()
    }
}
impl<T: AsULE + mdl::Write> mdl::WriteProperty for OptionalListRef<'_, T> {
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        if let Some(values) = self.0 {
            ListRef(values).write_mdl_property(name, writer)?;
        }
        Ok(())
    }
}
pub(super) struct UvSet(ZeroVec<'static, [f32; 2]>);
impl mdl::Read for UvSet {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        parser.expect_ident("TVertices")?;
        read_list(parser).map(Self)
    }
}
pub(super) struct UvSetsRef<'a>(&'a [ZeroVec<'a, [f32; 2]>]);
impl UvSetsRef<'_> {
    pub(super) fn iter(&self) -> impl Iterator<Item = UvRef<'_>> {
        self.0.iter().map(UvRef)
    }
}
pub(super) struct UvRef<'a>(&'a ZeroVec<'a, [f32; 2]>);
impl mdl::Write for UvRef<'_> {
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        ListRef(self.0).write_mdl_property("TVertices", writer)
    }
}
#[derive(Clone, Copy, mdl::Read, mdl::Write)]
#[mdl(block = "Anim")]
pub(super) struct AnimExtent {
    #[mdl(flatten)]
    extent: GeosetExtent,
}
pub(super) struct ExtentsRef<'a>(&'a ZeroVec<'a, GeosetExtent>);
impl ExtentsRef<'_> {
    pub(super) fn iter(&self) -> impl Iterator<Item = AnimExtent> + '_ {
        self.0.iter().map(|extent| AnimExtent { extent })
    }
}

pub(super) struct Faces {
    counts: ZeroVec<'static, u32>,
    indices: ZeroVec<'static, u16>,
}
pub(super) struct Groups {
    counts: ZeroVec<'static, u32>,
    indices: ZeroVec<'static, u32>,
}
fn check_count(parser: &Parser<'_>, expected: usize, actual: usize) -> Result<(), mdl::ReadError> {
    if expected != actual {
        Err(parser.error(mdl::ReadErrorKind::CountMismatch { expected, actual }))
    } else {
        Ok(())
    }
}
// Faces and Groups have two counts and nested variable-length index lists.
fn read_groups<T: mdl::Read + AsULE>(
    parser: &mut Parser<'_>,
    name: &'static str,
    triangles: bool,
) -> Result<(ZeroVec<'static, u32>, ZeroVec<'static, T>), mdl::ReadError> {
    let groups = parser.read::<u32>()? as usize;
    let total = parser.read::<u32>()? as usize;
    let mut counts = Vec::new();
    let mut indices = Vec::new();
    let mut body = parser.begin_block()?;
    while let Some(field) = body.next_field()? {
        if field.name != name {
            return Err(mdl::ReadError::new(
                field.span,
                mdl::ReadErrorKind::UnknownField,
            ));
        }
        let start = indices.len();
        if triangles {
            body.expect(TokenKind::OpenBrace)?;
            while !body.consume(TokenKind::CloseBrace)? {
                read_inline::<T>(&mut body, &mut indices)?;
                body.expect(TokenKind::Comma)?;
            }
            if (indices.len() - start) % 3 != 0 {
                return Err(body.error(mdl::ReadErrorKind::Expected("triangle index triples")));
            }
        } else {
            read_inline::<T>(&mut body, &mut indices)?;
            body.consume(TokenKind::Comma)?;
        }
        let count = u32::try_from(indices.len() - start)
            .map_err(|_| body.error(mdl::ReadErrorKind::InvalidNumber("u32 count")))?;
        counts.push(count.to_unaligned());
    }
    check_count(&body, groups, counts.len())?;
    check_count(&body, total, indices.len())?;
    body.finish()?;
    Ok((ZeroVec::new_owned(counts), ZeroVec::new_owned(indices)))
}
fn read_inline<T: mdl::Read + AsULE>(
    parser: &mut Parser<'_>,
    values: &mut Vec<T::ULE>,
) -> Result<(), mdl::ReadError> {
    parser.expect(TokenKind::OpenBrace)?;
    if parser.consume(TokenKind::CloseBrace)? {
        return Ok(());
    }
    loop {
        values.push(parser.read::<T>()?.to_unaligned());
        if parser.consume(TokenKind::CloseBrace)? {
            break;
        }
        parser.expect(TokenKind::Comma)?;
        if parser.consume(TokenKind::CloseBrace)? {
            break;
        }
    }
    Ok(())
}
impl mdl::ReadProperty for Faces {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        let (counts, indices) = read_groups(parser, "Triangles", true)?;
        Ok(Self { counts, indices })
    }
}
impl mdl::ReadProperty for Groups {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        let (counts, indices) = read_groups(parser, "Matrices", false)?;
        Ok(Self { counts, indices })
    }
}
pub(super) struct GroupsRef<'a, T: AsULE> {
    counts: &'a ZeroVec<'a, u32>,
    indices: &'a ZeroVec<'a, T>,
    triangles: bool,
}
impl<T: AsULE + mdl::Write> mdl::WriteProperty for GroupsRef<'_, T> {
    fn validate_mdl_property(&self, _: &'static str, _: Dialect) -> Result<(), mdl::WriteError> {
        let total = self
            .counts
            .iter()
            .try_fold(0usize, |total, count| total.checked_add(count as usize));
        if total != Some(self.indices.len())
            || self.counts.len() > u32::MAX as usize
            || self.indices.len() > u32::MAX as usize
        {
            return Err(mdl::WriteError::InvalidStructure {
                field: "mesh group counts",
            });
        }
        if self.triangles && self.counts.iter().any(|count| count % 3 != 0) {
            return Err(mdl::WriteError::InvalidStructure {
                field: "triangle index triples",
            });
        }
        Ok(())
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        self.validate_mdl_property(name, writer.dialect())?;
        writer.indent()?;
        writer.identifier(name)?;
        writer.formatted(format_args!(
            " {} {}",
            self.counts.len(),
            self.indices.len()
        ))?;
        writer.open_body()?;
        let mut offset = 0;
        for count in self.counts.iter() {
            if self.triangles {
                writer.begin_block("Triangles")?;
                writer.indent()?;
            } else {
                writer.indent()?;
                writer.raw("Matrices ")?;
            }
            writer.raw("{ ")?;
            for (index, value) in self
                .indices
                .iter()
                .skip(offset)
                .take(count as usize)
                .enumerate()
            {
                if index != 0 {
                    writer.raw(", ")?;
                }
                writer.write(&value)?;
            }
            writer.raw(" },\n")?;
            if self.triangles {
                writer.end_block()?;
            }
            offset += count as usize;
        }
        writer.end_block()
    }
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(fields, validate_read = "Self::validate")]
pub(super) struct Selection {
    #[mdl(flag = "Unselectable", default)]
    flag: bool,
    #[mdl(property = "SelectionFlags", delegate)]
    raw: RawSelection,
}
impl Selection {
    fn validate(&self, span: Span) -> Result<(), mdl::ReadError> {
        if self.flag && self.raw.0.is_some() {
            Err(mdl::ReadError::new(
                span,
                mdl::ReadErrorKind::DuplicateField,
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SkinRow<S: SkinWeight>(S);
impl<S: SkinWeight> AsULE for SkinRow<S> {
    type ULE = S::ULE;
    fn to_unaligned(self) -> Self::ULE {
        self.0.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        Self(S::from_unaligned(value))
    }
}
impl<S: SkinWeight> mdl::Read for SkinRow<S> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let values = if parser
            .peek()?
            .is_some_and(|token| token.kind == TokenKind::OpenBrace)
        {
            parser.read::<[u8; 8]>()?
        } else {
            let mut values = [0u8; 8];
            for (index, value) in values.iter_mut().enumerate() {
                if index != 0 {
                    parser.expect(TokenKind::Comma)?;
                }
                *value = parser.read()?;
            }
            values
        };
        let bone_indices = values[..4]
            .try_into()
            .map(|values: [u8; 4]| values.map(u16::from))
            .expect("four indices");
        let weights = values[4..]
            .try_into()
            .map(|values: [u8; 4]| values.map(u16::from))
            .expect("four weights");
        Ok(Self(
            S::from_influences(bone_indices, weights).expect("byte-sized MDL influences"),
        ))
    }
}
pub(super) struct SkinRef<'a, S: SkinWeight>(Option<&'a ZeroVec<'a, S>>);
impl<S: SkinWeight> mdl::WriteProperty for SkinRef<'_, S> {
    fn validate_mdl_property(&self, _: &'static str, _: Dialect) -> Result<(), mdl::WriteError> {
        if self.0.is_some_and(|rows| {
            rows.iter()
                .any(|row| row.bone_indices().iter().any(|&index| index > 255))
        }) {
            Err(mdl::WriteError::Unrepresentable {
                field: "skin bone index above 255",
            })
        } else if self.0.is_some_and(|rows| {
            rows.iter()
                .any(|row| row.weights().iter().any(|&weight| weight > 255))
        }) {
            Err(mdl::WriteError::Unrepresentable {
                field: "skin weight above 255",
            })
        } else {
            Ok(())
        }
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        self.validate_mdl_property(name, writer.dialect())?;
        if let Some(rows) = self.0 {
            writer.begin_counted_block(name, rows.len())?;
            for row in rows.iter() {
                writer.indent()?;
                if writer.dialect() == Dialect::HiveWorkshop {
                    writer.raw("{ ")?;
                }
                for (index, value) in row
                    .bone_indices()
                    .iter()
                    .copied()
                    .chain(row.weights())
                    .enumerate()
                {
                    if index != 0 {
                        writer.raw(", ")?;
                    }
                    writer.write(&value)?;
                }
                if writer.dialect() == Dialect::HiveWorkshop {
                    writer.raw(" }")?;
                }
                writer.raw(",\n")?;
            }
            writer.end_block()?;
        }
        Ok(())
    }
}

impl<'data, V: ModelVersion> Geoset<'data, V> {
    pub(super) fn mdl_positions(&self) -> ListRef<'_, Vec3> {
        ListRef(&self.vertices)
    }
    pub(super) fn set_mdl_positions(
        &mut self,
        value: List<Vec3>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.vertices = value.0;
        Ok(())
    }
    pub(super) fn mdl_directions(&self) -> ListRef<'_, Vec3> {
        ListRef(&self.normals)
    }
    pub(super) fn set_mdl_directions(
        &mut self,
        value: List<Vec3>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.normals = value.0;
        Ok(())
    }
    pub(super) fn mdl_vertex_indices(&self) -> UncountedRef<'_, u8> {
        UncountedRef(&self.vertex_groups)
    }
    pub(super) fn set_mdl_vertex_indices(
        &mut self,
        value: Uncounted<u8>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.vertex_groups = value.0;
        Ok(())
    }
    pub(super) fn mdl_uvs(&self) -> UvSetsRef<'_> {
        UvSetsRef(&self.uv_sets)
    }
    pub(super) fn set_mdl_uvs(
        &mut self,
        value: Vec<UvSet>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.uv_sets = value.into_iter().map(|set| set.0).collect();
        Ok(())
    }
    pub(super) fn mdl_bounds(&self) -> ExtentsRef<'_> {
        ExtentsRef(&self.sequence_extents)
    }
    pub(super) fn set_mdl_bounds(
        &mut self,
        value: ZeroVec<'data, AnimExtent>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.sequence_extents = value.cast();
        Ok(())
    }
    pub(super) fn mdl_triangles(&self) -> GroupsRef<'_, u16> {
        GroupsRef {
            counts: &self.primitive_counts,
            indices: &self.faces,
            triangles: true,
        }
    }
    pub(super) fn set_mdl_triangles(
        &mut self,
        value: Faces,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.primitive_types = (0..value.counts.len()).map(|_| 4u32).collect();
        self.primitive_counts = value.counts;
        self.faces = value.indices;
        Ok(())
    }
    pub(super) fn mdl_matrices(&self) -> GroupsRef<'_, u32> {
        GroupsRef {
            counts: &self.matrix_group_sizes,
            indices: &self.matrix_indices,
            triangles: false,
        }
    }
    pub(super) fn set_mdl_matrices(
        &mut self,
        value: Groups,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.matrix_group_sizes = value.counts;
        self.matrix_indices = value.indices;
        Ok(())
    }
    pub(super) fn mdl_selection(&self) -> Selection {
        Selection {
            flag: self.unselectable_raw == 4,
            raw: RawSelection(
                (!matches!(self.unselectable_raw, 0 | 4)).then_some(self.unselectable_raw),
            ),
        }
    }
    pub(super) fn set_mdl_selection(
        &mut self,
        value: Selection,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.unselectable_raw = value.raw.0.unwrap_or(if value.flag { 4 } else { 0 });
        Ok(())
    }
    pub(super) fn mdl_lod(&self) -> Option<u32> {
        self.level_of_detail.level_of_detail()
    }
    pub(super) fn mdl_lod_mut(&mut self) -> Option<&mut u32> {
        self.level_of_detail.level_of_detail_mut()
    }
    pub(super) fn mdl_lod_name(&self) -> LodNameRef<'_> {
        LodNameRef(self.level_of_detail.fixed_name())
    }
    pub(super) fn set_mdl_lod_name(
        &mut self,
        value: Option<FixedText<80>>,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        if let Some(value) = value {
            let target = self.level_of_detail.name_mut().ok_or(mdl::ReadError::new(
                span,
                mdl::ReadErrorKind::UnsupportedField,
            ))?;
            *target = value;
        }
        Ok(())
    }
    pub(super) fn mdl_tangents(&self) -> OptionalListRef<'_, [f32; 4]> {
        OptionalListRef(self.extra_sections.reforged().and_then(|storage| {
            storage.sections.iter().find_map(|section| match section {
                GeosetExtraSection::Tangents(values) => Some(values),
                _ => None,
            })
        }))
    }
    pub(super) fn set_mdl_tangents(
        &mut self,
        value: OptionalList<[f32; 4]>,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        if let Some(values) = value.0 {
            let storage = self
                .extra_sections
                .reforged_mut()
                .ok_or(mdl::ReadError::new(
                    span,
                    mdl::ReadErrorKind::UnsupportedField,
                ))?;
            storage.sections.push(GeosetExtraSection::Tangents(values));
        }
        Ok(())
    }
    pub(super) fn mdl_skin(&self) -> SkinRef<'_, V::SkinWeights> {
        SkinRef(self.extra_sections.reforged().and_then(|storage| {
            storage.sections.iter().find_map(|section| match section {
                GeosetExtraSection::Skin { weights } => Some(weights),
                _ => None,
            })
        }))
    }
    pub(super) fn set_mdl_skin(
        &mut self,
        value: OptionalList<SkinRow<V::SkinWeights>>,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        if let Some(values) = value.0 {
            let storage = self
                .extra_sections
                .reforged_mut()
                .ok_or(mdl::ReadError::new(
                    span,
                    mdl::ReadErrorKind::UnsupportedField,
                ))?;
            storage.sections.push(GeosetExtraSection::Skin {
                weights: values.cast(),
            });
        }
        Ok(())
    }
    fn mdl_lengths_match(&self) -> bool {
        let count = self.vertices.len();
        let skinned = self
            .try_skin_weights()
            .is_ok_and(|weights| weights.is_some_and(|rows| rows.len() == count));
        self.normals.len() == count
            && (self.vertex_groups.len() == count || (self.vertex_groups.is_empty() && skinned))
            && self.uv_sets.iter().all(|set| set.len() == count)
            && self.extra_sections.reforged().is_none_or(|storage| {
                storage.sections.iter().all(|section| match section {
                    GeosetExtraSection::Tangents(values) => values.len() == count,
                    GeosetExtraSection::Skin { weights } => weights.len() == count,
                })
            })
    }
    pub(super) fn validate_mdl_read(&self, span: Span) -> Result<(), mdl::ReadError> {
        if !self.mdl_lengths_match() {
            return Err(mdl::ReadError::new(
                span,
                mdl::ReadErrorKind::Expected(
                    "one normal, UV, tangent or skin row per vertex; vertex groups may be empty with complete skin weights",
                ),
            ));
        }
        Ok(())
    }
    pub(super) fn validate_mdl_write(&self) -> Result<(), mdl::WriteError> {
        if !self.mdl_lengths_match() {
            return Err(mdl::WriteError::Unrepresentable {
                field: "inconsistent per-vertex section lengths",
            });
        }
        if self.primitive_types.len() != self.primitive_counts.len()
            || self.primitive_types.iter().any(|kind| kind != 4)
        {
            return Err(mdl::WriteError::Unrepresentable {
                field: "non-triangle primitive groups",
            });
        }
        Ok(())
    }
}

pub(super) struct RawSelection(Option<u32>);
impl mdl::ReadProperty for RawSelection {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        parser.read_property().map(|value| Self(Some(value)))
    }
    fn missing_mdl_property(_: &'static str, _: Span) -> Result<Self, mdl::ReadError> {
        Ok(Self(None))
    }
}
impl mdl::WriteProperty for RawSelection {
    fn validate_mdl_property(
        &self,
        _: &'static str,
        dialect: Dialect,
    ) -> Result<(), mdl::WriteError> {
        if dialect == Dialect::Warcraft3 && self.0.is_some() {
            return Err(mdl::WriteError::Unrepresentable {
                field: "raw selection flags in Warcraft III dialect",
            });
        }
        Ok(())
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        self.validate_mdl_property(name, writer.dialect())?;
        if let Some(value) = self.0 {
            writer.property(name, &value)?;
        }
        Ok(())
    }
}
pub(super) struct LodNameRef<'a>(Option<&'a FixedText<80>>);
impl mdl::WriteProperty for LodNameRef<'_> {
    fn validate_mdl_property(&self, _: &'static str, _: Dialect) -> Result<(), mdl::WriteError> {
        if let Some(name) = self.0 {
            if *name != FixedText::default() {
                mdl::validate_fixed_text(name)?;
            }
        }
        Ok(())
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        self.validate_mdl_property(name, writer.dialect())?;
        if let Some(value) = self.0.filter(|value| **value != FixedText::default()) {
            writer.property(name, value)?;
        }
        Ok(())
    }
}

impl AsULE for AnimExtent {
    type ULE = <GeosetExtent as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        self.extent.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        Self {
            extent: GeosetExtent::from_unaligned(value),
        }
    }
}
