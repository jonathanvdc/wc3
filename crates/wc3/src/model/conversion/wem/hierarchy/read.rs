//! Parsed node rows are temporary decoder inputs; the published graph contains logical owners only.
use super::super::actors::common::Field;
use super::{
    base,
    payloads::{Decoded, ReadContext, ReadPayload},
    poses,
    setup::{RigField, SkinField},
};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::poses::{NodeSetup, Poses};
use crate::model::semantic::profiles::wem::{Availability, NodeControls};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{Node, NodeId};
use crate::model::wem::{self, records, RawDocument, RecordLayout, Ref};

pub(in crate::model::conversion::wem) trait SourceNode {
    fn has_feature(&self) -> bool;
    fn read_base<'view>(
        &self,
        document: &'view RawDocument<'_>,
        nodes: &[NodeId],
        poses: &Poses<'_>,
    ) -> Result<Node<'view, Wem>, Error>;
    fn read_payload<'view>(
        &self,
        context: &ReadContext<'_, 'view, '_>,
    ) -> Result<Decoded<'view>, Error>;
}
impl<D: records::NodeDialect> SourceNode for records::Node<D>
where
    D::Payload: ReadPayload,
    D::PoseMatrices: Field<Ref<records::targets::Matrix44>>,
    D::Skin: SkinField,
    D::Rig: RigField,
    D::Profiles: Field<u32>,
{
    fn has_feature(&self) -> bool {
        self.payload.has_feature()
    }
    fn read_base<'view>(
        &self,
        document: &'view RawDocument<'_>,
        nodes: &[NodeId],
        conventions: &Poses<'_>,
    ) -> Result<Node<'view, Wem>, Error> {
        let ids: Vec<_> = conventions.iter().map(|(id, _)| id).collect();
        let setup = NodeSetup {
            poses: poses::read_values(document, self, conventions)?,
            skin: self.skin.read_setup(document, nodes, &ids)?,
            rig: self.rig.read_setup(nodes)?,
        };
        setup.validate(conventions)?;
        let controls = NodeControls {
            setup,
            availability: Availability::from_bits_retain(self.profiles.read().unwrap_or(0)),
            ..Default::default()
        };
        base::read(document, self, nodes, controls)
    }
    fn read_payload<'view>(
        &self,
        context: &ReadContext<'_, 'view, '_>,
    ) -> Result<Decoded<'view>, Error> {
        self.payload.read_payload(context)
    }
}
fn typed<D: records::NodeDialect>(
    document: &RawDocument<'_>,
    reference: Ref<records::targets::Node>,
) -> Result<Vec<Box<dyn SourceNode>>, Error>
where
    records::Node<D>: SourceNode + RecordLayout + for<'record> wem::Read<'record> + 'static,
{
    poses::rows::<records::Node<D>, _>(document, reference).map(|rows| {
        rows.into_iter()
            .map(|value| Box::new(value) as Box<dyn SourceNode>)
            .collect()
    })
}
pub(in crate::model::conversion::wem) fn read(
    document: &RawDocument<'_>,
    reference: Ref<records::targets::Node>,
) -> Result<Vec<Box<dyn SourceNode>>, Error> {
    let Some(section) = document
        .referenced_section(reference)
        .map_err(|error| Error::new("wem.nodes", error.to_string()))?
    else {
        return Ok(Vec::new());
    };
    macro_rules! revisions{($($number:literal:$dialect:ident),+)=>{match section.revision(){$($number=>typed::<records::$dialect>(document,reference),)+_=>Err(Error::new("wem.nodes","unsupported node revision"))}};}
    revisions!(1:V1,2:V2,3:V3,4:V4,5:V5,6:V6,7:V7,8:V8,9:V9,10:V10,11:V11,12:V12,13:V13,14:V14,15:V15,16:V16,17:V17,18:V18,19:V19)
}
