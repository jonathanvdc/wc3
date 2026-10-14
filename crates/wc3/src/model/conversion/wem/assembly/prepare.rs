//! Collection identities and shared resources are reserved before model-local links resolve.
use super::super::{resources, text};
use super::records::{self, Model};
use crate::model::common::FileKey;
use crate::model::conversion::{
    Context, Error, Format, ImportOptions, IssueKind, OpaquePayload, UnknownPolicy,
};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{ModelCollectionBuilder, ModelId, TextureId};
use crate::model::wem::{
    self,
    records::{DocumentRecord, TextureRecord, V1},
    RawDocument, Ref,
};
use std::borrow::Cow;

pub(super) struct Prepared<'view> {
    pub root: DocumentRecord<V1>,
    pub models: Vec<Model>,
    pub builder: ModelCollectionBuilder<'view, Wem>,
    pub model_ids: Vec<ModelId>,
    pub textures: Vec<TextureId>,
    pub opaque: Vec<OpaquePayload<'view>>,
    pub context: Context,
}
fn error(value: impl ToString) -> Error {
    Error::new("wem", value.to_string())
}
pub(super) fn read<'view>(
    document: &'view RawDocument<'_>,
    options: &ImportOptions,
) -> Result<Prepared<'view>, Error> {
    if document.version() != 3 {
        return Err(error(
            "editable graph import requires the supported third-generation envelope",
        ));
    }
    document.validate().map_err(error)?;
    let source = document.root_reference();
    if source.entries != 1 {
        return Err(error("document root must identify exactly one declaration"));
    }
    let reference = Ref::<DocumentRecord<V1>>::from_section(source.entries, source.index);
    let root = document
        .resolve(reference)
        .map_err(error)?
        .get(0)
        .ok_or_else(|| error("missing document root"))?
        .map_err(error)?;
    let models = records::read(document, root.models)?;
    let mut context = Context::import(options);
    let mut opaque = Vec::new();
    for (_, section) in document.sections() {
        if matches!(section.tag(),tag if tag==*b"WOEM"||tag==*b"HOLE") {
            continue;
        }
        let span = wem::records::known_record_span(
            section.tag(),
            section.revision(),
            section.bytes(),
            section.count(),
            document.options().native_abi,
        )
        .map_err(error)?;
        if span.is_none() {
            match options.unknown {
                UnknownPolicy::Reject => {
                    return Err(Error::new(
                        "wem.section",
                        format!(
                            "unknown section {:?} revision {}",
                            section.tag(),
                            section.revision()
                        ),
                    ));
                }
                UnknownPolicy::Drop => context.issue(
                    "wem.section",
                    IssueKind::Dropped,
                    "unknown indexed payload omitted",
                ),
                UnknownPolicy::Archive => {
                    opaque.push(OpaquePayload {
                        format: Format::Wem,
                        file: FileKey::Root,
                        tag: section.tag(),
                        revision: Some(section.revision()),
                        count: Some(section.count()),
                        bytes: Cow::Borrowed(section.bytes()),
                    });
                    context.issue(
                        "wem.section",
                        IssueKind::Archived,
                        "unknown indexed payload retained outside the editable graph",
                    );
                }
            }
        } else {
            wem::records::validate_known_section(section.tag(),section.revision(),section.bytes(),section.count(),document.options().native_abi,&mut |reference|{
                if reference.flags.bits()!=0{context.issue("wem.reference",IssueKind::Normalized,"native reference qualifiers are not authored graph state; rebuilt references use canonical qualifiers");}
                Ok(())
            }).map_err(error)?;
        }
    }
    let mut builder = ModelCollectionBuilder::<Wem>::new();
    let mut model_ids = Vec::new();
    for source in &models {
        let index = builder.add_model(text::read(document, source.name)?);
        model_ids.push(
            builder
                .model_id(index)
                .ok_or_else(|| error("reserved model is absent"))?,
        );
    }
    let textures = document
        .resolve::<TextureRecord>(root.textures.cast())
        .map_err(error)?
        .iter()
        .map(|row| {
            Ok(builder
                .resources_mut()
                .add_texture(resources::read::<Wem>(document, &row.map_err(error)?)?))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(Prepared {
        root,
        models,
        builder,
        model_ids,
        textures,
        opaque,
        context,
    })
}
