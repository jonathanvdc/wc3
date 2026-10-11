//! Decoded and unknown model chunks.
use super::*;
use crate::model::Encoder;
use crate::model::ModelOwnership;
use crate::model::{mdx, Detached};
use crate::model::{Chunk, Cursor, KnownChunk, RawChunk};
use crate::model::{ModelDialect, Tag};
use mdx::Extension as _;
use std::borrow::Cow;
use std::marker::PhantomData;

/// An opaque chunk whose tag is not defined by this library.
#[derive(Clone, Debug, Eq, PartialEq, ModelOwnership)]
pub struct UnknownChunk<'data, D: ModelDialect> {
    raw: RawChunk<'data>,
    version: PhantomData<D>,
}

impl<'data, D: ModelDialect> UnknownChunk<'data, D> {
    /// Returns `None` when the tag belongs to a known chunk.
    pub fn new(raw: RawChunk<'data>) -> Option<Self> {
        (!ModelChunk::<D>::is_known_tag(raw.tag)).then_some(Self {
            raw,
            version: PhantomData,
        })
    }

    /// Borrows the opaque tag and payload.
    /// Moves the raw payload out of this unrecognized chunk.
    pub fn into_raw(self) -> RawChunk<'data> {
        self.raw
    }
    /// Borrows the retained raw chunk.
    pub fn raw(&self) -> &RawChunk<'data> {
        &self.raw
    }

    /// Edits the opaque payload without changing its checked tag.
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        self.raw.data.to_mut()
    }
}

// Keep the known variants, their tags, and their codecs in one place.
macro_rules! model_chunks {
    ($( $variant:ident($chunk:ty), )*) => {
        /// One ordered chunk in a model. Known variants contain complete decoded payloads.
        #[derive(Clone, Debug, ModelOwnership)]
        pub enum ModelChunk<'data, D: ModelDialect> {
            $( #[doc = concat!("Decoded `", stringify!($variant), "` chunk payload.")]
               $variant(Box<$chunk>), )*
            /// An unrecognized chunk retained as opaque bytes.
            Unknown(UnknownChunk<'data, D>),
            /// An application-defined decoded chunk.
            Extension(D::Extension),
        }

        $(
            impl<'data, D: ModelDialect> From<$chunk> for ModelChunk<'data, D> {
                fn from(chunk: $chunk) -> Self {
                    Self::$variant(Box::new(chunk))
                }
            }

            impl<'data, D: ModelDialect> TryFrom<ModelChunk<'data, D>> for $chunk {
                type Error = ModelChunk<'data, D>;

                fn try_from(chunk: ModelChunk<'data, D>) -> Result<Self, Self::Error> {
                    match chunk {
                        ModelChunk::$variant(value) => Ok(*value),
                        other => Err(other),
                    }
                }
            }

            impl<'data, 'a, D: ModelDialect> TryFrom<&'a ModelChunk<'data, D>> for &'a $chunk {
                type Error = ();

                fn try_from(chunk: &'a ModelChunk<'data, D>) -> Result<Self, Self::Error> {
                    match chunk {
                        ModelChunk::$variant(value) => Ok(value.as_ref()),
                        _ => Err(()),
                    }
                }
            }
        )*

        $(
            impl<'data, 'a, D: ModelDialect> TryFrom<&'a mut ModelChunk<'data, D>> for &'a mut $chunk {
                type Error = ();
                fn try_from(chunk: &'a mut ModelChunk<'data, D>) -> Result<Self, Self::Error> {
                    match chunk { ModelChunk::$variant(value) => Ok(value.as_mut()), _ => Err(()) }
                }
            }
        )*

        impl<'data, D: ModelDialect> Chunk for ModelChunk<'data, D> where D::Extension: Chunk {
            fn tag(&self) -> Tag { ModelChunk::tag(self) }
            fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
                match self {
                    $( Self::$variant(value) => value.encode_payload_to(output), )*
                    Self::Unknown(unknown) => unknown.raw.encode_payload_to(output),
                    Self::Extension(extension) => {
                        if Self::is_known_tag(extension.tag()) {
                            return Err(mdx::WriteError::InvalidValue { tag: extension.tag(), field: "extension uses a standard chunk tag" });
                        }
                        extension.encode_payload_to(output)
                    },
                }
            }
        }

        impl<'data, D: ModelDialect> ModelChunk<'data, D> {
            fn is_known_tag(tag: Tag) -> bool {
                matches!(tag, $( <$chunk>::TAG )|*)
            }

            fn decode_payload(
                tag: Tag,
                payload: &mut Cursor<'data>,
            ) -> Result<Option<Self>, mdx::ReadError> where D::Extension: mdx::Extension<'data> {
                let mut cursor = *payload;
                let decoded = match tag {
                    $( <$chunk>::TAG => <$chunk>::decode_payload(&mut cursor).map(Self::from), )*
                    _ => {
                        let extension = D::Extension::read_extension(tag, &mut cursor)
                            .map_err(|error| error.in_chunk(tag))?;
                        if let Some(extension) = extension {
                            cursor.finish().map_err(|error| error.in_chunk(tag))?;
                            if extension.tag() != tag {
                                return Err(mdx::ReadError::new(payload.absolute_position(), mdx::ReadErrorKind::InvalidValue { field: "extension changed its chunk tag" }).in_chunk(tag));
                            }
                            return Ok(Some(Self::Extension(extension)));
                        }
                        return Ok(None);
                    },
                };
                let chunk = decoded.map_err(|error| error.in_chunk(tag))?;
                cursor.finish().map_err(|error| error.in_chunk(tag))?;
                Ok(Some(chunk))
            }

            pub(crate) fn standard_tag(&self) -> Option<Tag> {
                match self {
                    $( Self::$variant(_) => Some(<$chunk>::TAG), )*
                    Self::Unknown(unknown) => Some(unknown.raw.tag),
                    Self::Extension(_) => None,
                }
            }

            /// Returns this chunk's tag.
            pub fn tag(&self) -> Tag where D::Extension: Chunk {
                match self {
                    $( Self::$variant(_) => <$chunk>::TAG, )*
                    Self::Unknown(unknown) => unknown.raw.tag,
                    Self::Extension(extension) => extension.tag(),
                }
            }
        }
    };
}

model_chunks! {
    Version(VersionChunk<'data, D::Version>),
    ModelInfo(ModelInfoChunk<'data>),
    Sequences(SequencesChunk),
    GlobalSequences(GlobalSequencesChunk),
    Textures(TexturesChunk),
    Materials(MaterialsChunk<'data, D::Version>),
    Geosets(GeosetsChunk<'data, D::Version>),
    GeosetAnimations(GeosetAnimationsChunk<'data>),
    Bones(BonesChunk<'data>),
    Helpers(HelpersChunk<'data>),
    Attachments(AttachmentsChunk<'data>),
    EventObjects(EventObjectsChunk<'data>),
    CollisionShapes(CollisionShapesChunk<'data>),
    ParticleEmitters(ParticleEmittersChunk<'data>),
    ParticleEmitters2(ParticleEmitters2Chunk<'data>),
    RibbonEmitters(RibbonEmittersChunk<'data>),
    PopcornEmitters(PopcornEmittersChunk<'data>),
    Cameras(CamerasChunk<'data, D::Version>),
    Lights(LightsChunk<'data, D::Version>),
    TextureAnimations(TextureAnimationsChunk<'data>),
    FaceFx(FaceFxChunk),
    PivotPoints(PivotPointsChunk),
    BindPose(BindPoseChunk),
    Gliders(GlidersChunk),
}

impl<'data, D: ModelDialect> ModelChunk<'data, D>
where
    D::Extension: mdx::Extension<'data>,
{
    /// Decodes standard and application chunks, retaining unrecognized payloads.
    /// Borrows a raw chunk's payload while decoding its records.
    pub fn from_raw_ref(raw: &'data RawChunk<'_>) -> Result<Self, mdx::ReadError> {
        Self::decode_from(raw.tag, &mut Cursor::new(&raw.data))
    }

    pub(crate) fn decode_from(
        tag: Tag,
        payload: &mut Cursor<'data>,
    ) -> Result<Self, mdx::ReadError> {
        match Self::decode_payload(tag, payload) {
            Ok(Some(chunk)) => Ok(chunk),
            Ok(None) => Ok(Self::Unknown(UnknownChunk {
                raw: RawChunk::new(tag, payload.remaining_bytes()),
                version: PhantomData,
            })),
            Err(error) => Err(error),
        }
    }
}

impl<'data, D: ModelDialect + 'static> ModelChunk<'data, D>
where
    D::Extension: for<'input> mdx::Extension<'input>,
{
    /// Decodes a raw chunk, moving opaque bytes and detaching decoded owned payloads.
    pub fn from_raw(raw: RawChunk<'data>) -> Result<Self, mdx::ReadError> {
        let tag = raw.tag;
        match raw.data.into_cow() {
            Cow::Borrowed(bytes) => Self::decode_from(tag, &mut Cursor::new(bytes)),
            Cow::Owned(bytes) => {
                let mut input = Cursor::new(&bytes);
                let decoded: Option<Self> =
                    ModelChunk::<D>::decode_payload(tag, &mut input)?.map(Detached::into_detached);
                match decoded {
                    Some(chunk) => Ok(chunk),
                    None => Ok(Self::Unknown(UnknownChunk {
                        raw: RawChunk::new(tag, bytes),
                        version: PhantomData,
                    })),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::V800;

    #[test]
    fn typed_chunks_convert_in_both_directions() {
        let version = VersionChunk::<V800>::new();
        let chunk: ModelChunk<'_, V800> = version.clone().into();
        assert!(matches!(chunk, ModelChunk::Version(_)));
        assert_eq!(<&VersionChunk<'_, V800>>::try_from(&chunk), Ok(&version));
        assert_eq!(VersionChunk::<V800>::try_from(chunk).unwrap(), version);
    }

    #[test]
    fn failed_extraction_preserves_the_chunk() {
        let chunk = ModelChunk::Unknown(
            UnknownChunk::<V800>::new(RawChunk::new(*b"FUTR", vec![1, 2, 3])).unwrap(),
        );
        assert!(<&VersionChunk<'_, V800>>::try_from(&chunk).is_err());
        let original = VersionChunk::<V800>::try_from(chunk).unwrap_err();
        assert!(matches!(original, ModelChunk::Unknown(raw) if raw.raw().data == [1, 2, 3]));
    }
}
