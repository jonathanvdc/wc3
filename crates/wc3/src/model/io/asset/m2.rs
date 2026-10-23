//! Runtime M2 dialect dispatch; native bundles remain responsible for the closure.
use super::{CompanionKind, DecodeError, Error, Request};
use crate::model::common::FileKey;
use crate::model::conversion::{Conversion, ImportOptions};
use crate::model::formats::m2::{self as native, DependencyKind};
use crate::model::io::m2 as semantic;
use crate::model::Model;
use std::collections::BTreeMap;

fn request(dependency: &native::Dependency) -> Request {
    Request {
        owner: dependency.owner.clone(),
        key: dependency.key.clone(),
        kind: match dependency.kind {
            DependencyKind::Skin { index, lod } => CompanionKind::Skin { index, lod },
            DependencyKind::Animation {
                animation_id,
                variation_id,
            } => CompanionKind::Animation {
                animation_id,
                variation_id,
            },
            DependencyKind::Skeleton => CompanionKind::Skeleton,
            DependencyKind::Bone => CompanionKind::Bone,
            DependencyKind::Physics => CompanionKind::Physics,
            DependencyKind::Document => CompanionKind::Model,
            DependencyKind::Texture => unreachable!("textures are not native bundle companions"),
        },
    }
}

fn decode_typed<'a, D: native::Dialect>(
    bytes: &'a [u8],
    key: &FileKey,
    buffers: &'a BTreeMap<FileKey, Vec<u8>>,
    missing: &mut Option<Request>,
) -> Result<Option<native::Bundle<'a, D>>, native::ReadError> {
    match native::Bundle::decode(bytes, key.clone(), |dependency| {
        if let Some(bytes) = buffers.get(&dependency.key) {
            Ok(Some(bytes.as_slice()))
        } else {
            *missing = Some(request(dependency));
            Ok(None)
        }
    }) {
        Ok(bundle) => Ok(Some(bundle)),
        Err(_) if missing.is_some() => Ok(None),
        Err(error) => Err(error),
    }
}

#[derive(Debug)]
pub(super) struct Preparation {
    layout: Layout,
    dependencies: Vec<Request>,
}

macro_rules! bundles {
    ($($variant:ident : $envelope:ident<$layout:ident>),* $(,)?) => {
        #[derive(Debug)]
        enum Layout { $($variant,)* }
        pub(super) fn prepare(bytes: &[u8], key: &FileKey) -> Result<Preparation, native::ReadError> {
            let document = native::DynamicDocument::decode_m2(bytes)?;
            let dependencies = document.dependencies(key).iter()
                .filter(|dependency| !matches!(dependency.kind, DependencyKind::Texture))
                .map(request).collect();
            let layout = match document { $(native::DynamicDocument::$variant(_) => Layout::$variant,)* };
            Ok(Preparation { layout, dependencies })
        }
        pub(super) fn decode(
            prepared: &Preparation,
            bytes: &[u8],
            key: &FileKey,
            buffers: &BTreeMap<FileKey, Vec<u8>>,
            missing: &mut Option<Request>,
            options: &ImportOptions,
        ) -> Result<Option<Conversion<Model<'static>>>, Error> {
            // Prefetch directly declared companions before decoding the full bundle.
            // Recursive companion requirements remain native bundle decisions.
            if let Some(request) = prepared.dependencies.iter().find(|request| !buffers.contains_key(&request.key)) {
                *missing = Some(request.clone());
                return Ok(None);
            }
            match prepared.layout {
                $(Layout::$variant => decode_typed::<native::$envelope<native::$layout>>(bytes, key, buffers, missing)
                    .map_err(|source| Error::Decode { key: key.clone(), source: DecodeError::M2(source) })?
                    .map(|bundle| semantic::import(&bundle, options).map(|converted| Conversion {
                        value: converted.value.widen().into_owned(),
                        report: converted.report,
                    }).map_err(|source| Error::Import { key: key.clone(), source }))
                    .transpose(),)*
            }
        }
    };
}
bundles! {
    Flat256: Flat<V256>, Chunked256: Chunked<V256>,
    Flat257: Flat<V257>, Chunked257: Chunked<V257>,
    Flat260: Flat<V260>, Chunked260: Chunked<V260>,
    Flat263: Flat<V263>, Chunked263: Chunked<V263>,
    Flat264: Flat<V264>, Chunked264: Chunked<V264>,
    Flat265: Flat<V265>, Chunked265: Chunked<V265>,
    Flat272: Flat<V272>, Chunked272: Chunked<V272>,
    Flat272PreSplit: Flat<V272PreSplit>, Chunked272PreSplit: Chunked<V272PreSplit>,
    Flat273: Flat<V273>, Chunked273: Chunked<V273>,
    Flat274: Flat<V274>, Chunked274: Chunked<V274>,
}
