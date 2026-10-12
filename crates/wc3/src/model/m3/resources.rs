//! Texture dependencies declared by native material fields.
use super::records::{
    targets, DataDrivenMaterial, StringReference, TextureLayer, V0, V1, V10, V11, V12, V13, V14,
    V15, V16, V17, V18, V19, V2, V20, V21, V22, V23, V24, V25, V26, V3, V4, V5, V6, V7, V8, V9,
};
use super::{Document, Envelope, Model, Read, ReadError, Record, RecordLayout, Ref, SectionId};
use crate::model::common::FileKey;
use std::str::from_utf8;

/// Native material field declaring a texture dependency.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextureOwner {
    /// A LAYR texture layer's texture-path field.
    Layer {
        /// Indexed LAYR section containing the field.
        section: SectionId,
        /// Record index in the section.
        record: usize,
    },
    /// A MADD material's SCHR texture-path table.
    DataDriven {
        /// Indexed MADD section containing the material.
        section: SectionId,
        /// Material record index.
        record: usize,
        /// String-reference index in the material's texture table.
        slot: usize,
    },
}
/// A texture path borrowed from its native CHAR section.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextureDependency<'view> {
    /// Native field that declares the texture dependency.
    pub owner: TextureOwner,
    /// Native reference, including the encoded terminator in its entry count.
    pub reference: Ref<targets::Character>,
    /// Filename bytes with the final null terminator removed.
    pub path: &'view [u8],
}
impl TextureDependency<'_> {
    /// Returns an independent resolver key without changing path separators.
    pub fn key(&self) -> FileKey {
        match from_utf8(self.path) {
            Ok(path) => FileKey::Path(path.to_owned()),
            Err(_) => FileKey::RawPath(self.path.to_vec()),
        }
    }
}
impl<E: Envelope> Document<'_, E> {
    /// Enumerates paths declared by understood LAYR and MADD record revisions.
    ///
    /// Shared paths retain each declaring field's identity. Empty references and
    /// empty strings contribute no dependency. Unknown revisions remain opaque;
    /// names and other CHAR references are not interpreted as texture paths.
    pub fn texture_dependencies(&self) -> Result<Vec<TextureDependency<'_>>, ReadError> {
        let mut dependencies = Vec::new();
        for (section, value) in self.sections() {
            match (&value.tag(), value.revision()) {
                (b"LAYR", 0) => {
                    self.layer_paths::<TextureLayer<V0>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 1) => {
                    self.layer_paths::<TextureLayer<V1>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 2) => {
                    self.layer_paths::<TextureLayer<V2>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 3) => {
                    self.layer_paths::<TextureLayer<V3>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 4) => {
                    self.layer_paths::<TextureLayer<V4>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 5) => {
                    self.layer_paths::<TextureLayer<V5>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 6) => {
                    self.layer_paths::<TextureLayer<V6>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 7) => {
                    self.layer_paths::<TextureLayer<V7>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 8) => {
                    self.layer_paths::<TextureLayer<V8>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 9) => {
                    self.layer_paths::<TextureLayer<V9>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 10) => {
                    self.layer_paths::<TextureLayer<V10>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 11) => {
                    self.layer_paths::<TextureLayer<V11>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 12) => {
                    self.layer_paths::<TextureLayer<V12>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 13) => {
                    self.layer_paths::<TextureLayer<V13>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 14) => {
                    self.layer_paths::<TextureLayer<V14>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 15) => {
                    self.layer_paths::<TextureLayer<V15>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 16) => {
                    self.layer_paths::<TextureLayer<V16>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 17) => {
                    self.layer_paths::<TextureLayer<V17>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 18) => {
                    self.layer_paths::<TextureLayer<V18>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 19) => {
                    self.layer_paths::<TextureLayer<V19>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 20) => {
                    self.layer_paths::<TextureLayer<V20>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 21) => {
                    self.layer_paths::<TextureLayer<V21>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 22) => {
                    self.layer_paths::<TextureLayer<V22>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 23) => {
                    self.layer_paths::<TextureLayer<V23>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 24) => {
                    self.layer_paths::<TextureLayer<V24>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 25) => {
                    self.layer_paths::<TextureLayer<V25>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"LAYR", 26) => {
                    self.layer_paths::<TextureLayer<V26>>(section, &mut dependencies, |v| {
                        v.texture_path
                    })?
                }
                (b"MADD", 0) => self.material_paths::<DataDrivenMaterial<V0>>(
                    section,
                    &mut dependencies,
                    |v| v.texture_paths,
                )?,
                (b"MADD", 1) => self.material_paths::<DataDrivenMaterial<V1>>(
                    section,
                    &mut dependencies,
                    |v| v.texture_paths,
                )?,
                (b"MADD", 2) => self.material_paths::<DataDrivenMaterial<V2>>(
                    section,
                    &mut dependencies,
                    |v| v.texture_paths,
                )?,
                (b"MADD", 3) => self.material_paths::<DataDrivenMaterial<V3>>(
                    section,
                    &mut dependencies,
                    |v| v.texture_paths,
                )?,
                _ => {}
            }
        }
        Ok(dependencies)
    }
    fn layer_paths<'view, T>(
        &'view self,
        section: SectionId,
        dependencies: &mut Vec<TextureDependency<'view>>,
        path: impl Fn(T) -> Ref<targets::Character>,
    ) -> Result<(), ReadError>
    where
        T: Record + RecordLayout<E> + for<'data> Read<'data, E>,
    {
        for (record, value) in self.array::<T>(section)?.iter().enumerate() {
            self.texture_path(
                TextureOwner::Layer { section, record },
                path(value?),
                dependencies,
            )?;
        }
        Ok(())
    }
    fn material_paths<'view, T>(
        &'view self,
        section: SectionId,
        dependencies: &mut Vec<TextureDependency<'view>>,
        paths: impl Fn(T) -> Ref<targets::StringReference>,
    ) -> Result<(), ReadError>
    where
        T: Record + RecordLayout<E> + for<'data> Read<'data, E>,
    {
        for (record, value) in self.array::<T>(section)?.iter().enumerate() {
            let strings = self.resolve::<StringReference>(paths(value?).cast())?;
            for (slot, string) in strings.iter().enumerate() {
                self.texture_path(
                    TextureOwner::DataDriven {
                        section,
                        record,
                        slot,
                    },
                    string?.value,
                    dependencies,
                )?;
            }
        }
        Ok(())
    }
    fn texture_path<'view>(
        &'view self,
        owner: TextureOwner,
        reference: Ref<targets::Character>,
        dependencies: &mut Vec<TextureDependency<'view>>,
    ) -> Result<(), ReadError> {
        let Some(section) = self.referenced_section(reference)? else {
            return Ok(());
        };
        if section.revision() != 0 {
            return Err(ReadError::invalid(0, "texture string revision"));
        }
        let bytes = section
            .bytes()
            .get(..reference.entries as usize)
            .ok_or_else(|| ReadError::invalid(0, "texture string span"))?;
        let path = bytes
            .strip_suffix(&[0])
            .ok_or_else(|| ReadError::invalid(0, "texture string terminator"))?;
        if !path.is_empty() {
            dependencies.push(TextureDependency {
                owner,
                reference,
                path,
            });
        }
        Ok(())
    }
}

impl<E: Envelope> Model<'_, E> {
    /// Enumerates native material texture paths from this model's document.
    pub fn texture_dependencies(&self) -> Result<Vec<TextureDependency<'_>>, ReadError> {
        self.document().texture_dependencies()
    }
}
