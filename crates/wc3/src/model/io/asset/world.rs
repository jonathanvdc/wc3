//! Explicit modern WMO layout dispatch and required group loading.
use super::{CompanionKind, DecodeError, Error, Request};
use crate::model::common::FileKey;
use crate::model::conversion::{self, Conversion, ImportOptions};
use crate::model::formats::wmo::{self as native, Profile};
use crate::model::io::wmo as semantic;
use crate::model::Model;
use std::collections::BTreeMap;

fn decode_typed<D: native::Dialect>(
    bytes: &[u8],
    key: &FileKey,
    buffers: &BTreeMap<FileKey, Vec<u8>>,
    missing: &mut Option<Request>,
) -> Result<Option<native::Bundle<'static, D>>, Error> {
    let failure = |key: &FileKey, source| Error::Decode {
        key: key.clone(),
        source: DecodeError::Wmo(source),
    };
    let root = native::Root::<D>::decode(bytes).map_err(|source| failure(key, source))?;
    let mut bundle = native::Bundle::from_root(root).with_key(key.clone());
    let dependencies = bundle
        .dependencies()
        .map_err(|source| failure(key, source))?;
    for dependency in &dependencies {
        let native::DependencyKind::Group { index, lod } = dependency.kind else {
            continue;
        };
        if !buffers.contains_key(&dependency.key) {
            *missing = Some(Request {
                owner: key.clone(),
                key: dependency.key.clone(),
                kind: CompanionKind::WmoGroup { index, lod },
            });
            return Ok(None);
        }
    }
    // Native geometry is decoded only after every declared group is available.
    // Waiting for a later group must not repeatedly decode earlier groups.
    for dependency in dependencies {
        let native::DependencyKind::Group { index, lod } = dependency.kind else {
            continue;
        };
        let bytes = &buffers[&dependency.key];
        let group =
            native::Group::<D>::decode(bytes).map_err(|source| failure(&dependency.key, source))?;
        bundle.insert_group(native::GroupKey { index, lod }, dependency.key, group);
    }
    bundle
        .validate_complete()
        .map_err(|source| failure(key, source))?;
    Ok(Some(bundle.into_owned()))
}

macro_rules! bundles {
    ($($dialect:ident),* $(,)?) => {
        #[derive(Debug)]
        pub(super) enum Bundle {
            $($dialect(Box<native::Bundle<'static, native::$dialect>>),)*
        }
        impl Bundle {
            pub(super) fn import(&self, options: &ImportOptions) -> Result<Conversion<Model<'_>>, conversion::Error> {
                match self {
                    $(Self::$dialect(bundle) => semantic::import(bundle.as_ref(), options).map(|c| Conversion { value: c.value.widen(), report: c.report }),)*
                }
            }
        }
        pub(super) fn decode(
            bytes: &[u8],
            key: &FileKey,
            profile: Profile,
            buffers: &BTreeMap<FileKey, Vec<u8>>,
            missing: &mut Option<Request>,
        ) -> Result<Option<Bundle>, Error> {
            match profile {
                $(Profile::$dialect => decode_typed::<native::$dialect>(bytes, key, buffers, missing)
                    .map(|bundle| bundle.map(|bundle| Bundle::$dialect(Box::new(bundle)))),)*
                Profile::Alpha14 | Profile::Legacy16 => Err(Error::UnsupportedProfile { profile }),
            }
        }
    };
}
bundles! { Classic, Legion, Bfa, ShadowlandsPre92, Shadowlands, Dragonflight, WarWithin }
