use wc3::model::animation::{Sequence, SequenceFlags};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::Model;

#[test]
fn sequence_fields_round_trip() {
    let mut stand = Sequence::new("Stand", [0, 1000]).unwrap();
    stand.move_speed = 270.0;
    let mut flags = SequenceFlags::default();
    flags.set_non_looping(true);
    stand.flags = flags;
    stand.rarity = 0.5;
    stand.sync_point = 500;
    stand.extent.bounds_radius = 42.0;
    stand.extent.minimum = [-2.0, -3.0, -4.0];
    stand.extent.maximum = [2.0, 3.0, 4.0];
    let mut model = Model::<wc3::model::V800>::new();
    model.set_sequences(&[stand]);
    let decoded = Model::<wc3::model::V800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let sequence = &decoded.sequences()[0];
    assert_eq!(sequence.name.text(), "Stand");
    assert_eq!(sequence.interval, [0, 1000]);
    assert_eq!(sequence.move_speed, 270.0);
    assert!(sequence.flags.non_looping());
    assert_eq!(sequence.rarity, 0.5);
    assert_eq!(sequence.sync_point, 500);
    assert_eq!(sequence.extent.bounds_radius, 42.0);
    assert_eq!(sequence.extent.minimum, [-2.0, -3.0, -4.0]);
    assert_eq!(sequence.extent.maximum, [2.0, 3.0, 4.0]);
}
