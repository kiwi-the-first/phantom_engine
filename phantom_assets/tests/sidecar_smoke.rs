// Throwaway smoke test for the full-name sidecar convention + legacy migration.
use std::path::PathBuf;

use phantom_assets::asset_manager::{AssetManager, AssetType, PhantomAsset};
use uuid::Uuid;

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("phantom_sidecar_test_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn new_sidecars_use_full_file_name() {
    let project = temp_project();
    let src_dir = temp_project();
    let png = src_dir.join("player.png");
    std::fs::write(&png, b"fake png").unwrap();
    let wav = src_dir.join("player.wav");
    std::fs::write(&wav, b"fake wav").unwrap();

    let mut manager = AssetManager::default();
    manager.init(&project).unwrap();
    manager.import_asset(png, project.clone()).unwrap();
    manager.import_asset(wav, project.clone()).unwrap();

    assert!(project.join("player.png.passet").is_file());
    assert!(project.join("player.wav.passet").is_file());
    // The old colliding stem-style name must not appear.
    assert!(!project.join("player.passet").exists());

    // Lookup goes through the same convention.
    let (_, asset_type) = manager
        .find_uuid_and_asset_type(&project.join("player.png"))
        .unwrap();
    assert!(asset_type == AssetType::Sprite);
    let (_, asset_type) = manager
        .find_uuid_and_asset_type(&project.join("player.wav"))
        .unwrap();
    assert!(asset_type == AssetType::Audio);

    std::fs::remove_dir_all(project).unwrap();
    std::fs::remove_dir_all(src_dir).unwrap();
}

#[test]
fn init_migrates_legacy_stem_sidecars() {
    let project = temp_project();
    let png = project.join("player.png");
    std::fs::write(&png, b"fake png").unwrap();

    let id = Uuid::new_v4();
    let legacy = PhantomAsset::new(id, AssetType::Sprite, png.clone());
    std::fs::write(
        project.join("player.passet"),
        serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();

    let mut manager = AssetManager::default();
    manager.init(&project).unwrap();

    assert!(project.join("player.png.passet").is_file());
    assert!(!project.join("player.passet").exists());
    // UUID survives the migration.
    assert!(manager.find_sprite_by_id(&id).is_some());

    std::fs::remove_dir_all(project).unwrap();
}
