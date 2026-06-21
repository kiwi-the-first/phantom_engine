use std::{
    fs,
    path::{Path, PathBuf},
    slice::Windows,
};

use phantom_assets::asset_manager::{AssetManager, PhantomAsset, asset_manager};
use phantom_common::dirs;
use phantom_core::{
    ecs::{World, components::Sprite},
    serde_json,
};
use phantom_project::project_manager::project_manager::ProjectManager;
#[derive(Default)]
pub struct BuildSystem;

impl BuildSystem {
    pub fn build(
        &self,
        asset_manager: &mut AssetManager,
        project_path: PathBuf,
    ) -> anyhow::Result<()> {
        let (project, mut world) = ProjectManager::load(project_path.clone())?;

        BuildSystem::create_build_dirs(&project_path)?;

        let asset_paths = BuildSystem::collect_sprite_assets(asset_manager);
        BuildSystem::copy_assets(&asset_paths, &project_path, asset_manager)?;
        //BuildSystem::update_sprite_paths(asset_manager)?;
        let world_bytes = world.serialize();
        fs::write(
            dirs::BuildDirs::data(&project_path).join(project.entry_world.file_name().unwrap()),
            world_bytes,
        )?;

        BuildSystem::generate_lib_rs(&project_path.join("src"))?;
        BuildSystem::compile_dylib(&project_path, &project.name)?;

        Ok(())
    }
    fn create_build_dirs(build_path: &PathBuf) -> anyhow::Result<()> {
        let assets_path = dirs::BuildDirs::assets(&build_path);
        log::trace!("Creating directories {}", assets_path.to_string_lossy());
        std::fs::create_dir_all(assets_path)?;
        Ok(())
    }
    fn collect_sprite_assets(asset_manager: &mut AssetManager) -> Vec<PathBuf> {
        let sprite_assets = asset_manager.grab_all_registered_sprite_assets();
        sprite_assets.iter().map(|s| s.get_asset_path()).collect()
    }
    fn copy_assets(
        assets: &[PathBuf],
        project_path: &PathBuf,
        asset_manager: &mut AssetManager,
    ) -> anyhow::Result<()> {
        for asset_path in assets {
            let source = project_path.join(asset_path);
            let passet_path = AssetManager::passet_path_for(&source);

            let file = std::fs::read(passet_path)?;
            let mut passet: PhantomAsset = serde_json::from_slice(&file)?;

            let file_name = source.file_name().ok_or(anyhow::anyhow!(
                "invalid asset path: {}",
                asset_path.to_string_lossy()
            ))?;
            passet.set_asset_path(PathBuf::from("assets").join(file_name));

            let dest = dirs::BuildDirs::assets(&project_path).join(file_name);

            let passet_dest = AssetManager::passet_path_for(&dest);
            let passet_json = serde_json::to_vec(&passet)?;
            fs::write(&passet_dest, passet_json)?;

            std::fs::copy(&source, &dest)?;

            log::trace!(
                "Copied {} to {}",
                source.to_string_lossy(),
                dest.to_string_lossy()
            );
        }
        Ok(())
    }
    fn update_sprite_paths(asset_manager: &mut AssetManager) -> anyhow::Result<()> {
        let assets = asset_manager.grab_all_registered_sprite_assets();
        for asset in assets {
            let file_name = asset
                .get_asset_path()
                .file_name()
                .clone()
                .unwrap()
                .to_string_lossy()
                .to_string();

            asset.set_asset_path(PathBuf::from(format!("assets/{}", file_name)));
        }

        // let entities = world.query_with::<Sprite>();
        // for entity in entities {
        //     let sprite = world.get_component_mut::<Sprite>(entity).unwrap();
        //     if !sprite.asset.is_empty() {
        //         let file_name = PathBuf::from(&sprite.asset)
        //             .file_name()
        //             .unwrap()
        //             .to_string_lossy()
        //             .to_string();
        //         sprite.asset = format!("assets/{}", file_name);
        //     }
        // }
        Ok(())
    }

    fn collect_registerable_types(src_dir: &Path) -> anyhow::Result<Vec<(String, String)>> {
        let mut types = Vec::new();
        Self::scan_dir(src_dir, src_dir, &mut types)?;
        types.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(types)
    }

    fn scan_dir(
        src_dir: &Path,
        dir: &Path,
        types: &mut Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::scan_dir(src_dir, &path, types)?;
                continue;
            }

            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            if path.file_name().and_then(|n| n.to_str()) == Some("lib.rs") {
                continue;
            }
            if path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let lines: Vec<&str> = content.lines().collect();

            for i in 0..lines.len() {
                let trimmed = lines[i].trim();
                if trimmed == "#[script]" || trimmed == "#[component]" {
                    for j in i + 1..lines.len() {
                        if let Some(struct_name) = Self::parse_struct_name(lines[j]) {
                            let rel = path.strip_prefix(src_dir).unwrap();
                            let module_path = rel
                                .with_extension("")
                                .components()
                                .map(|c| c.as_os_str().to_str().unwrap().to_string())
                                .collect::<Vec<_>>()
                                .join("::");
                            types.push((module_path, struct_name));
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_struct_name(line: &str) -> Option<String> {
        let mut parts = line.split_whitespace();
        while let Some(word) = parts.next() {
            if word == "struct" {
                return parts
                    .next()
                    .map(|s| s.trim_end_matches('{').trim().to_string());
            }
        }
        None
    }

    fn generate_lib_rs(src_dir: &Path) -> anyhow::Result<()> {
        let types = Self::collect_registerable_types(src_dir)?;

        let mut out = String::new();
        out.push_str("use phantom_core::phantom_macros::phantom_register;\n\n");

        let mut top_level_modules: Vec<&str> = types
            .iter()
            .map(|(m, _)| m.split("::").next().unwrap())
            .collect();
        top_level_modules.dedup();
        for module in &top_level_modules {
            out.push_str(&format!("pub mod {};\n", module));
        }

        if !types.is_empty() {
            out.push_str("\nphantom_register!(\n");
            for (module_path, struct_name) in &types {
                out.push_str(&format!("    {}::{},\n", module_path, struct_name));
            }
            out.push_str(");\n");
        } else {
            out.push_str("\nphantom_register!();\n");
        }

        fs::write(src_dir.join("lib.rs"), out)?;
        log::trace!("Generated lib.rs with {} type(s)", types.len());
        Ok(())
    }

    fn compile_dylib(project_path: &PathBuf, project_name: &String) -> anyhow::Result<()> {
        let status = std::process::Command::new("cargo")
            .args(["build", "--release", "--manifest-path"])
            .arg(project_path.join("Cargo.toml"))
            .current_dir(project_path)
            .status()?;

        if !status.success() {
            anyhow::bail!("cargo build failed with exit code {:?}", status.code());
        }

        #[cfg(windows)]
        let dylib_name = format!("{}.dll", project_name);
        #[cfg(target_os = "macos")]
        let dylib_name = format!("lib{}.dylib", project_name);
        #[cfg(target_os = "linux")]
        let dylib_name = format!("lib{}.so", project_name);

        let src = project_path.join("target/release").join(&dylib_name);
        let dest = dirs::BuildDirs::data(project_path).join(&dylib_name);
        std::fs::copy(src, dest)?;
        Ok(())
    }
}
