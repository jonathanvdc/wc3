use super::*;
#[test]
fn model_paths_preserve_locations_and_support_mdl_fallback() {
    let model = Path::new("units/parent.mdx");
    assert_eq!(
        model_paths(model, "Effects\\Spark.MDL"),
        [
            "units/Effects/Spark.MDL",
            "units/Effects/Spark.mdx",
            "Effects/Spark.MDL",
            "Effects/Spark.mdx",
        ]
    );
    assert_eq!(
        model_paths(model, "Spark.mdx"),
        ["units/Spark.mdx", "Spark.mdx"]
    );
    assert_eq!(
        model_paths(Path::new("parent.mdl"), "Spark.mdl"),
        ["Spark.mdl", "Spark.mdx"]
    );
    for path in ["", "../secret.mdl", "/secret.mdx", "Spark.tga"] {
        assert!(model_paths(model, path).is_empty());
    }
}

#[test]
fn texture_paths_try_all_extensions_locally_before_asset_root() {
    assert_eq!(
        texture_paths(Path::new("units/footman.mdx"), "Textures\\armor.TIF"),
        [
            "units/Textures/armor.TIF",
            "units/Textures/armor.blp",
            "units/Textures/armor.dds",
            "units/Textures/armor.png",
            "units/Textures/armor.tga",
            "Textures/armor.TIF",
            "Textures/armor.blp",
            "Textures/armor.dds",
            "Textures/armor.png",
            "Textures/armor.tga",
        ]
    );
}

#[test]
fn texture_paths_deduplicate_literal_extension_and_locations() {
    assert_eq!(
        texture_paths(Path::new("footman.mdl"), "body.png"),
        ["body.png", "body.blp", "body.dds", "body.tga"]
    );
    assert_eq!(
        texture_paths(Path::new("footman.mdl"), "body"),
        ["body", "body.blp", "body.dds", "body.png", "body.tga"]
    );
}

#[test]
fn texture_paths_reject_invalid_paths() {
    for name in [
        "",
        "../secret.blp",
        "/secret.tif",
        "Textures\\..\\secret.tif",
    ] {
        assert!(texture_paths(Path::new("units/footman.mdx"), name).is_empty());
    }
}
