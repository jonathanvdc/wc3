use super::super::super::text;
use super::super::{
    initial::{self, Initial},
    routing,
};
use super::codec::{append, error, read_words, words};
use crate::model::conversion::Error;
use crate::model::semantic::{
    profiles::wem::animation::{
        AnyChannelId, AnyDeclaration, Channels, Declaration, Selection, SelectionId, Selections,
    },
    TextureId,
};
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::HashMap;
pub(super) fn channels(
    document: &RawDocument<'_>,
    reference: Ref<records::targets::AnimChannel>,
    context: &routing::ReadContext<'_, '_>,
    textures: &[TextureId],
) -> Result<(Channels, HashMap<u32, AnyChannelId>), Error> {
    let mut channels = Channels::default();
    let mut ids = HashMap::new();
    for row in document
        .resolve::<records::AnimChannel>(reference.cast())
        .map_err(error)?
        .iter()
    {
        let row = row.map_err(error)?;
        if ids.contains_key(&row.id) {
            return Err(Error::new(
                "wem.program.channel",
                "duplicate channel identity",
            ));
        }
        let texture_selection = matches!(
            row.target.channel,
            records::Channel::TextureIndex
                | records::Channel::NormalTextureIndex
                | records::Channel::OrmTextureIndex
                | records::Channel::EmissiveTextureIndex
                | records::Channel::TeamColorTextureIndex
                | records::Channel::EnvironmentTextureIndex
        );
        macro_rules! declare {
            ($value:expr,$variant:ident) => {{
                let target = routing::read(&row.target, context)?;
                AnyChannelId::$variant(channels.add(Declaration {
                    target,
                    initial: $value,
                }))
            }};
        }
        let id = match initial::read(document, &row, textures, texture_selection)? {
            Initial::Scalar(value) => declare!(value, Scalar),
            Initial::Vector2(value) => declare!(value, Vector2),
            Initial::Vector3(value) => declare!(value, Vector3),
            Initial::Vector4(value) => declare!(value, Vector4),
            Initial::Rotation(value) => declare!(value, Rotation),
            Initial::Boolean(value) => declare!(value, Boolean),
            Initial::Unsigned16(value) => declare!(value, Unsigned16),
            Initial::Unsigned(value) => declare!(value, Unsigned),
            Initial::Signed(value) => declare!(value, Signed),
            Initial::Bytes4(value) => declare!(value, Bytes4),
            Initial::Unsigned4(value) => declare!(value, Unsigned4),
            Initial::Texture(value) => declare!(value, Texture),
        };
        ids.insert(row.id, id);
    }
    Ok((channels, ids))
}
pub(super) fn selections<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::TrackSet>,
    channels: &Channels,
    ids: &HashMap<u32, AnyChannelId>,
) -> Result<(Selections<'a>, Vec<SelectionId>), Error> {
    let mut selections = Selections::default();
    let mut keys = Vec::new();
    for row in document
        .resolve::<records::TrackSet>(reference.cast())
        .map_err(error)?
        .iter()
    {
        let row = row.map_err(error)?;
        let selected = read_words(document, row.channels)?
            .iter()
            .map(|id| {
                ids.get(&id).copied().ok_or_else(|| {
                    Error::new(
                        "wem.program.selection",
                        "selection contains an undeclared channel",
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        keys.push(selections.add(
            Selection {
                name: text::read(document, row.name)?,
                channels: selected,
            },
            channels,
        )?);
    }
    Ok((selections, keys))
}

pub(super) struct Written {
    pub channels: Ref<records::targets::AnimChannel>,
    pub selections: Ref<records::targets::TrackSet>,
    pub ids: HashMap<AnyChannelId, u32>,
    pub selection_ids: HashMap<SelectionId, u32>,
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    channels: &Channels,
    selections: &Selections<'_>,
    routing: &routing::WriteContext<'_, '_>,
    textures: &std::collections::BTreeMap<TextureId, u32>,
) -> Result<Written, Error> {
    let mut ids = HashMap::new();
    let mut rows = Vec::new();
    for (ordinal, declaration) in channels.declarations().enumerate() {
        let id = u32::try_from(ordinal).map_err(error)?;
        ids.insert(declaration.id(), id);
        let target = match declaration {
            AnyDeclaration::Scalar(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Vector2(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Vector3(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Vector4(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Rotation(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Boolean(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Unsigned16(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Unsigned(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Signed(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Bytes4(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Unsigned4(_, value) => routing::write(&value.target, routing)?,
            AnyDeclaration::Texture(_, value) => routing::write(&value.target, routing)?,
        };
        let (value_type, init_value) = initial::write(document, declaration, textures)?;
        rows.push(records::AnimChannel {
            id,
            target,
            value_type,
            init_value,
        });
    }
    let channels = append(document, &rows)?.cast();
    let mut selection_ids = HashMap::new();
    let mut rows = Vec::new();
    for (ordinal, (id, selection)) in selections.iter().enumerate() {
        selection_ids.insert(id, u32::try_from(ordinal).map_err(error)?);
        let name = text::write_bytes(document, selection.name.as_bytes())?;
        let channels = words(
            document,
            selection
                .channels
                .iter()
                .map(|id| {
                    ids.get(id)
                        .copied()
                        .ok_or_else(|| error("selection names a foreign declaration"))
                })
                .collect::<Result<Vec<_>, Error>>()?,
        )?;
        rows.push(records::TrackSet { name, channels });
    }
    let selections = append(document, &rows)?.cast();
    Ok(Written {
        channels,
        selections,
        ids,
        selection_ids,
    })
}
