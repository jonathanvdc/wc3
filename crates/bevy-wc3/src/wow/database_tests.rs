use super::*;
use crate::wow::{Customization, Equipment, GeosetSelection};
use wc3::model::ResourceLocator;
fn tables() -> CharacterTables {
    CharacterTables {
        bodies: BTreeMap::from([(
            (4, Sex::Female),
            CharacterDefinition {
                model: ResourceLocator::FileDataId(1),
                geosets: GeosetSelection {
                    visible: BTreeSet::from([0, 101, 401]),
                },
                textures: BTreeMap::new(),
                parts: Vec::new(),
            },
        )]),
        ..Default::default()
    }
}
#[test]
fn explicit_hide_rules_and_priorities_ignore_request_order() {
    let mut database = tables();
    let patch = |order, visible| AppearancePatch {
        order,
        geosets: vec![GeosetRule {
            alternatives: BTreeSet::from([401, 402, 403]),
            visible: BTreeSet::from([visible]),
        }],
        ..Default::default()
    };
    database
        .equipment
        .insert((4, Sex::Female, EquipmentSlot::Chest, 10), patch(13, 402));
    database
        .equipment
        .insert((4, Sex::Female, EquipmentSlot::Hands, 20), patch(20, 403));
    database.customization.insert(
        (4, Sex::Female, 1, 2),
        AppearancePatch {
            geosets: vec![GeosetRule {
                alternatives: BTreeSet::from([101]),
                visible: BTreeSet::new(),
            }],
            ..Default::default()
        },
    );
    let mut recipe = CharacterRecipe {
        race: 4,
        sex: Sex::Female,
        customization: vec![Customization {
            option: 1,
            choice: 2,
        }],
        equipment: vec![
            Equipment {
                slot: EquipmentSlot::Hands,
                display: 20,
            },
            Equipment {
                slot: EquipmentSlot::Chest,
                display: 10,
            },
        ],
    };
    assert_eq!(
        recipe.resolve(&database).unwrap().geosets.visible,
        BTreeSet::from([0, 403])
    );
    recipe.equipment.reverse();
    assert_eq!(
        recipe.resolve(&database).unwrap().geosets.visible,
        BTreeSet::from([0, 403])
    );
}
#[test]
fn absent_exact_race_and_sex_rows_do_not_guess_a_substitute() {
    let database = tables();
    let recipe = CharacterRecipe {
        race: 4,
        sex: Sex::Male,
        customization: Vec::new(),
        equipment: Vec::new(),
    };
    assert!(recipe.resolve(&database).is_err());
}
