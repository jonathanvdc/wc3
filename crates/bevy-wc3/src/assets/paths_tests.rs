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
fn texture_paths_try_model_then_asset_root() {
    let model = Path::new("units/footman.mdx");
    assert_eq!(
        texture_paths(model, "body.blp"),
        ["units/body.blp", "body.blp"]
    );
    assert_eq!(
        texture_paths(model, "Textures\\armor.blp"),
        ["units/Textures/armor.blp", "Textures/armor.blp"]
    );
    assert!(texture_paths(model, "../secret.blp").is_empty());
}
