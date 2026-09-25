//! Light records in `LITE` chunks.
use crate::{Color, Tag, Version};

use crate::Record;
use crate::{AnimationTrack, Error, Model, Node};
use crate::{Cursor, LightsChunk, ModelChunk};

const FIXED_SIZE: usize = 44;
const EXTENDED_SIZE: usize = 72;

/// A light node with decoded lighting values and animation tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    node: Node,
    light_type: u32,
    attenuation_start: f32,
    attenuation_end: f32,
    color: Color,
    intensity: f32,
    ambient_color: Color,
    ambient_intensity: f32,
    extended_words: Option<[u32; 7]>,
    tracks: Vec<AnimationTrack>,
}

impl Light {
    /// Creates a light with zeroed lighting values.
    pub fn new(node: Node, light_type: u32) -> Self {
        Self {
            node,
            light_type,
            attenuation_start: 0.0,
            attenuation_end: 0.0,
            color: [0.0; 3],
            intensity: 0.0,
            ambient_color: [0.0; 3],
            ambient_intensity: 0.0,
            extended_words: None,
            tracks: Vec::new(),
        }
    }

    /// Creates a light with the additional fixed fields used by newer models.
    pub fn new_for_version(node: Node, light_type: u32, version: Version) -> Self {
        let mut light = Self::new(node, light_type);
        if version >= 1200 {
            light.extended_words = Some([0; 7]);
        }
        light
    }

    /// Borrows the shared node.
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Borrows the shared node for editing.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    /// Returns raw light type ID.
    pub fn light_type(&self) -> u32 {
        self.light_type
    }
    /// Sets raw light type ID.
    pub fn set_light_type(&mut self, kind: u32) {
        self.light_type = kind;
    }
    /// Returns attenuation start distance.
    pub fn attenuation_start(&self) -> f32 {
        self.attenuation_start
    }
    /// Sets attenuation start distance.
    pub fn set_attenuation_start(&mut self, value: f32) {
        self.attenuation_start = value;
    }
    /// Returns attenuation end distance.
    pub fn attenuation_end(&self) -> f32 {
        self.attenuation_end
    }
    /// Sets attenuation end distance.
    pub fn set_attenuation_end(&mut self, value: f32) {
        self.attenuation_end = value;
    }
    /// Returns RGB light color.
    pub fn color(&self) -> Color {
        self.color
    }
    /// Sets RGB light color.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    /// Returns light intensity.
    pub fn intensity(&self) -> f32 {
        self.intensity
    }
    /// Sets light intensity.
    pub fn set_intensity(&mut self, value: f32) {
        self.intensity = value;
    }
    /// Returns ambient RGB color.
    pub fn ambient_color(&self) -> Color {
        self.ambient_color
    }
    /// Sets ambient RGB color.
    pub fn set_ambient_color(&mut self, color: Color) {
        self.ambient_color = color;
    }
    /// Returns ambient intensity.
    pub fn ambient_intensity(&self) -> f32 {
        self.ambient_intensity
    }
    /// Sets ambient intensity.
    pub fn set_ambient_intensity(&mut self, value: f32) {
        self.ambient_intensity = value;
    }
    /// Returns the additional seven raw words in newer light records, when present.
    pub fn extended_words(&self) -> Option<[u32; 7]> {
        self.extended_words
    }
    /// Sets the additional seven raw words in a newer light record.
    pub fn set_extended_words(&mut self, words: [u32; 7]) -> Result<(), Error> {
        if self.extended_words.is_none() {
            return Err(Error::MalformedRecord {
                tag: Light::TAG,
                offset: 4 + self.node.encode()?.len() + FIXED_SIZE,
            });
        }
        self.extended_words = Some(words);
        Ok(())
    }
    /// Borrows decoded light animation tracks.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }
    /// Replaces optional light animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        for track in tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Light::TAG,
                    offset: 0,
                });
            }
            track.encode()?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track(tag: Tag) -> bool {
    matches!(
        &tag,
        b"KLAV" | b"KLAC" | b"KLAI" | b"KLBC" | b"KLBI" | b"KLAS" | b"KLAE"
    )
}

impl Model {
    /// Decodes all `LITE` records in file order.
    pub fn lights(&self) -> Result<Vec<Light>, Error> {
        self.collect_chunk_records::<LightsChunk>(|chunk| match chunk {
            ModelChunk::Lights(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces lights in the first `LITE` chunk.
    pub fn set_lights(&mut self, lights: &[Light]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::Lights(LightsChunk::new(lights.to_vec())))
    }
}

impl Record for Light {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let node = Node::decode_one(&mut cursor, 0)?;
        let light_type = cursor.read_u32()?;
        let attenuation_start = cursor.read_f32()?;
        let attenuation_end = cursor.read_f32()?;
        let color = cursor.read_vec3()?;
        let intensity = cursor.read_f32()?;
        let ambient_color = cursor.read_vec3()?;
        let ambient_intensity = cursor.read_f32()?;
        let remaining = cursor.remaining();
        let extension_size = EXTENDED_SIZE - FIXED_SIZE;
        let has_extended = remaining.len() >= extension_size
            && (remaining.len() == extension_size
                || remaining
                    .get(extension_size..extension_size + 4)
                    .is_some_and(|tag| is_track(tag.try_into().expect("four-byte tag"))))
            && remaining.get(..4).map_or(true, |tag| {
                !is_track(tag.try_into().expect("four-byte tag"))
            });
        let extended_words = if has_extended {
            let mut words = [0; 7];
            for word in &mut words {
                *word = cursor.read_u32()?;
            }
            Some(words)
        } else {
            None
        };
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Self::TAG,
                    offset,
                });
            }

            tracks.push(track);
        }
        cursor.finish()?;
        Ok(Self {
            node,
            light_type,
            attenuation_start,
            attenuation_end,
            color,
            intensity,
            ambient_color,
            ambient_intensity,
            extended_words,
            tracks,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(&self.node.encode()?);
        bytes.extend_from_slice(&self.light_type.to_le_bytes());
        bytes.extend_from_slice(&self.attenuation_start.to_le_bytes());
        bytes.extend_from_slice(&self.attenuation_end.to_le_bytes());
        for value in self.color {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&self.intensity.to_le_bytes());
        for value in self.ambient_color {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&self.ambient_intensity.to_le_bytes());
        if let Some(words) = self.extended_words {
            for word in words {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Light::TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.encode()?);
        }
        let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
            tag: Light::TAG,
            size: bytes.len(),
        })?;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
    }
}

impl Light {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"LITE";
}
