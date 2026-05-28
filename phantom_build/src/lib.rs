use std::{
    fs,
    path::{Path, PathBuf},
};

use phantom_common::dirs;
use phantom_core::ecs::{World, components::Sprite};
use phantom_project::project_manager::project_manager::ProjectManager;

pub struct BuildSystem;

impl BuildSystem {
    pub fn build(project_path: PathBuf) -> anyhow::Result<()> {
        let (project, mut world) = ProjectManager::load(project_path.clone())?;

        BuildSystem::create_build_dirs(&project_path)?;

        let asset_paths = BuildSystem::collect_sprite_assets(&world);
        BuildSystem::copy_assets(&asset_paths, &project_path)?;
        BuildSystem::update_sprite_paths(&mut world)?;
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
    fn collect_sprite_assets(world: &World) -> Vec<String> {
        world
            .query_with::<Sprite>()
            .iter()
            .map(|entity| {
                world
                    .get_component::<Sprite>(*entity)
                    .unwrap()
                    .asset_path
                    .clone()
            })
            .filter(|path| !path.is_empty())
            .collect()
    }
    fn copy_assets(assets: &[String], project_path: &PathBuf) -> anyhow::Result<()> {
        for asset_path in assets {
            let source = project_path.join(asset_path);
            let file_name = source
                .file_name()
                .ok_or(anyhow::anyhow!("invalid asset path: {}", asset_path))?;
            let dest = dirs::BuildDirs::assets(&project_path).join(file_name);
            std::fs::copy(&source, &dest)?;
            log::trace!(
                "Copied {} to {}",
                source.to_string_lossy(),
                dest.to_string_lossy()
            );
        }
        Ok(())
    }
    fn update_sprite_paths(world: &mut World) -> anyhow::Result<()> {
        let entities = world.query_with::<Sprite>();
        for entity in entities {
            let sprite = world.get_component_mut::<Sprite>(entity).unwrap();
            if !sprite.asset_path.is_empty() {
                let file_name = PathBuf::from(&sprite.asset_path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                sprite.asset_path = format!("assets/{}", file_name);
            }
        }
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
        std::process::Command::new("cargo")
            .args(["build", "--release", "--manifest-path"])
            .arg(project_path.join("Cargo.toml"))
            .current_dir(project_path)
            .status()?;

        let dylib_name = format!("lib{}.so", project_name);
        let src = project_path.join("target/release").join(&dylib_name);
        let dest = dirs::BuildDirs::data(project_path).join(&dylib_name);
        std::fs::copy(src, dest)?;
        Ok(())
    }
}
