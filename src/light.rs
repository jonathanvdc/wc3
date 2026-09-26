//! Light records in `LITE` chunks.
use crate::Encoder;
use crate::ValueError;
use crate::{Color, Tag, Version};

use crate::{AnimationTrack, Error, Model, Node};
use crate::{Cursor, LightsChunk};
use crate::{Decodable, Encodable};

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
    pub fn set_extended_words(&mut self, words: [u32; 7]) -> Result<(), ValueError> {
        if self.extended_words.is_none() {
            return Err(ValueError::UnavailableField {
                tag: Light::TAG,
                field: "extended words",
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
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), ValueError> {
        for track in tracks {
            if !is_track(track.tag) {
                return Err(ValueError::InvalidTrackTag {
                    record: Light::TAG,
                    track: track.tag,
                });
            }
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
    pub fn lights(&self) -> Vec<Light> {
        self.collect_chunk_records::<LightsChunk>()
    }

    /// Replaces lights in the first `LITE` chunk.
    pub fn set_lights(&mut self, lights: &[Light]) {
        self.replace_chunk(LightsChunk::new(lights.to_vec()));
    }
}

impl Decodable for Light {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let node = Node::decode_one(&mut cursor, 0)?;
        let light_type = cursor.read()?;
        let attenuation_start = cursor.read()?;
        let attenuation_end = cursor.read()?;
        let color = cursor.read_vector()?;
        let intensity = cursor.read()?;
        let ambient_color = cursor.read_vector()?;
        let ambient_intensity = cursor.read()?;
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
                *word = cursor.read()?;
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
}

impl Encodable for Light {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        self.node.encode_to(bytes)?;
        bytes.write(self.light_type);
        bytes.write(self.attenuation_start);
        bytes.write(self.attenuation_end);
        for value in self.color {
            bytes.write(value);
        }
        bytes.write(self.intensity);
        for value in self.ambient_color {
            bytes.write(value);
        }
        bytes.write(self.ambient_intensity);
        if let Some(words) = self.extended_words {
            for word in words {
                bytes.write(word);
            }
        }
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Light::TAG,
                    offset: bytes.position() - start,
                });
            }
            track.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, Light::TAG)?;
        Ok(())
    }
}

impl Light {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"LITE";
}
