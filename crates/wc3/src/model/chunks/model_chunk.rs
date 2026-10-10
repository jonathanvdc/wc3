//! Decoded and unknown model chunks.
use super::*;
use crate::model::mdx;
use crate::model::Encoder;
use crate::model::{Chunk, Cursor, KnownChunk, RawChunk};
use crate::model::{ModelDialect, Tag};
use mdx::Extension as _;
use std::marker::PhantomData;

/// An opaque chunk whose tag is not defined by this library.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownChunk<D: ModelDialect> {
    raw: RawChunk,
    version: PhantomData<D>,
}

impl<D: ModelDialect> UnknownChunk<D> {
    /// Returns `None` when the tag belongs to a known chunk.
    pub fn new(raw: RawChunk) -> Option<Self> {
        (!ModelChunk::<D>::is_known_tag(raw.tag)).then_some(Self {
            raw,
            version: PhantomData,
        })
    }

    /// Borrows the opaque tag and payload.
    pub fn raw(&self) -> &RawChunk {
        &self.raw
    }

    /// Edits the opaque payload without changing its checked tag.
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.raw.data
    }
}

// Keep the known variants, their tags, and their codecs in one place.
macro_rules! model_chunks {
    ($( $variant:ident($chunk:ty), )*) => {
        /// One ordered chunk in a model. Known variants contain complete decoded payloads.
        #[derive(Clone, Debug)]
        pub enum ModelChunk<D: ModelDialect> {
            $( #[doc = concat!("Decoded `", stringify!($variant), "` chunk payload.")]
               $variant(Box<$chunk>), )*
            /// An unrecognized chunk retained as opaque bytes.
            Unknown(UnknownChunk<D>),
            /// An application-defined decoded chunk.
            Extension(D::Extension),
        }

        $(
            impl<D: ModelDialect> From<$chunk> for ModelChunk<D> {
                fn from(chunk: $chunk) -> Self {
                    Self::$variant(Box::new(chunk))
                }
            }

            impl<D: ModelDialect> TryFrom<ModelChunk<D>> for $chunk {
                type Error = ModelChunk<D>;

                fn try_from(chunk: ModelChunk<D>) -> Result<Self, Self::Error> {
                    match chunk {
                        ModelChunk::$variant(value) => Ok(*value),
                        other => Err(other),
                    }
                }
            }

            impl<'a, D: ModelDialect> TryFrom<&'a ModelChunk<D>> for &'a $chunk {
                type Error = ();

                fn try_from(chunk: &'a ModelChunk<D>) -> Result<Self, Self::Error> {
                    match chunk {
                        ModelChunk::$variant(value) => Ok(value.as_ref()),
                        _ => Err(()),
                    }
                }
            }
        )*

        impl<D: ModelDialect> Chunk for ModelChunk<D> where D::Extension: Chunk {
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

        impl<D: ModelDialect> ModelChunk<D> {
            fn is_known_tag(tag: Tag) -> bool {
                matches!(tag, $( <$chunk>::TAG )|*)
            }

            fn decode_payload(
                tag: Tag,
                payload: &mut Cursor<'_>,
            ) -> Result<Option<Self>, mdx::ReadError> where D::Extension: mdx::Extension {
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
    Version(VersionChunk<D::Version>),
    ModelInfo(ModelInfoChunk),
    Sequences(SequencesChunk),
    GlobalSequences(GlobalSequencesChunk),
    Textures(TexturesChunk),
    Materials(MaterialsChunk<D::Version>),
    Geosets(GeosetsChunk<D::Version>),
    GeosetAnimations(GeosetAnimationsChunk),
    Bones(BonesChunk),
    Helpers(HelpersChunk),
    Attachments(AttachmentsChunk),
    EventObjects(EventObjectsChunk),
    CollisionShapes(CollisionShapesChunk),
    ParticleEmitters(ParticleEmittersChunk),
    ParticleEmitters2(ParticleEmitters2Chunk),
    RibbonEmitters(RibbonEmittersChunk),
    PopcornEmitters(PopcornEmittersChunk),
    Cameras(CamerasChunk<D::Version>),
    Lights(LightsChunk<D::Version>),
    TextureAnimations(TextureAnimationsChunk),
    FaceFx(FaceFxChunk),
    PivotPoints(PivotPointsChunk),
    BindPose(BindPoseChunk),
    Gliders(GlidersChunk),
}

impl<D: ModelDialect> ModelChunk<D>
where
    D::Extension: mdx::Extension,
{
    /// Decodes standard and application chunks, retaining unrecognized payloads.
    pub fn from_raw(raw: RawChunk) -> Result<Self, mdx::ReadError> {
        let decoded = Self::decode_payload(raw.tag, &mut Cursor::new(&raw.data));
        match decoded {
            Ok(Some(chunk)) => Ok(chunk),
            Ok(None) => Ok(Self::Unknown(UnknownChunk {
                raw,
                version: PhantomData,
            })),
            Err(error) => Err(error),
        }
    }

    /// Decodes directly from a bounded payload, copying bytes only when they
    /// must be retained for an unknown chunk.
    pub(crate) fn decode_from(tag: Tag, payload: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        match Self::decode_payload(tag, payload) {
            Ok(Some(chunk)) => Ok(chunk),
            Ok(None) => Ok(Self::Unknown(UnknownChunk {
                raw: RawChunk::new(tag, payload.remaining().to_vec()),
                version: PhantomData,
            })),
            Err(error) => Err(error),
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
        let chunk: ModelChunk<V800> = version.clone().into();
        assert!(matches!(chunk, ModelChunk::Version(_)));
        assert_eq!(<&VersionChunk<V800>>::try_from(&chunk), Ok(&version));
        assert_eq!(VersionChunk::<V800>::try_from(chunk).unwrap(), version);
    }

    #[test]
    fn failed_extraction_preserves_the_chunk() {
        let chunk = ModelChunk::Unknown(
            UnknownChunk::<V800>::new(RawChunk::new(*b"FUTR", vec![1, 2, 3])).unwrap(),
        );
        assert!(<&VersionChunk<V800>>::try_from(&chunk).is_err());
        let original = VersionChunk::<V800>::try_from(chunk).unwrap_err();
        assert!(matches!(original, ModelChunk::Unknown(raw) if raw.raw().data == [1, 2, 3]));
    }
}
