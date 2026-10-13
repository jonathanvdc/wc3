use super::*;
use wem::records as wr;

#[test]
fn draw_controls_and_uv_transforms_are_editable_native_fields() {
    let source = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut doc = source.model.into_document();
    let mut division = doc
        .resolve::<nr::MeshDivision<nr::V2>>(root.divisions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut region = doc
        .resolve::<nr::Region<nr::V5>>(division.regions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut batch = doc
        .resolve::<nr::Batch<nr::V1>>(division.batches.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    region.index = 41;
    region.unknown = 0xdeadbeef;
    region.bone_count = 1;
    region.padding = 0xabc;
    region.unknown_byte = 7;
    region.flags = 0x1234;
    region.uv_scale = 32.;
    region.uv_offset = 0.5;
    batch.flags = 0x321;
    batch.priority_plane = 9;
    batch.bounds_index = 13;
    batch.color_index = 17;
    batch.visibility_bone = 0;
    division.instances = 3;
    doc.replace(doc.section_id(division.regions.index).unwrap(), &[region])
        .unwrap();
    doc.replace(doc.section_id(division.batches.index).unwrap(), &[batch])
        .unwrap();
    doc.replace(doc.section_id(root.divisions.index).unwrap(), &[division])
        .unwrap();
    let imported = conversion::m3::import(
        &m3::Bundle::from_model(m3::Model::from_document(doc).unwrap()),
        &ImportOptions::default(),
    )
    .unwrap()
    .value;
    let mut semantic = imported.model(&ExportOptions::default()).unwrap();
    assert_eq!(semantic.meshes[0].uv_sets[0][1], [2.5, 0.5]);
    semantic.meshes[0].uv_sets[0].to_mut()[1] = [1.5, 0.5];
    let mut raw = wem::semantic::build(&semantic).unwrap().into_raw();
    let root = raw
        .resolve::<wr::DocumentRecord<wr::V1>>(wem::Ref::from_section(
            raw.root_reference().entries,
            raw.root_reference().index,
        ))
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let model = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mesh = raw
        .resolve::<wr::Mesh<wr::V2>>(model.meshes.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let section = raw
        .resolve::<wr::MeshSection>(mesh.sections.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut entries = raw
        .resolve::<wr::NativeEntry<wr::V2>>(section.native.entries.cast())
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let index = entries
        .iter()
        .position(|e| raw.string(e.name).unwrap() == Some("m3.batch.priority_plane"))
        .unwrap();
    entries[index].value = 25;
    raw.replace(
        raw.section_id(section.native.entries.index).unwrap(),
        &entries,
    )
    .unwrap();
    let edited = Asset::new(wem::Document::from_raw(raw).unwrap());
    let exported = conversion::m3::plan::<m3::StarCraft2>(&edited, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = exported.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let doc = exported.model.document();
    let division = doc
        .resolve::<nr::MeshDivision<nr::V2>>(root.divisions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let region = doc
        .resolve::<nr::Region<nr::V5>>(division.regions.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let batch = doc
        .resolve::<nr::Batch<nr::V1>>(division.batches.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            region.index,
            region.unknown,
            region.padding,
            region.unknown_byte,
            region.flags
        ),
        (41, 0xdeadbeef, 0xabc, 7, 0x1234)
    );
    assert_eq!((region.uv_scale, region.uv_offset), (32., 0.5));
    assert_eq!(
        (
            batch.flags,
            batch.priority_plane,
            batch.bounds_index,
            batch.color_index,
            batch.visibility_bone
        ),
        (0x321, 25, 13, 17, 0)
    );
    assert_eq!(division.instances, 3);
    let reread = conversion::m3::import(&exported, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        reread.model(&ExportOptions::default()).unwrap().meshes[0].uv_sets[0][1],
        [1.5, 0.5]
    );
}
