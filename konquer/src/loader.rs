use serde;
use serde_json;
use serde::{Deserialize, Serialize};
use glob;

use std::{fs, collections::HashMap, iter::zip};

use bevy::prelude::*;

use crate::*;

pub struct AssetLoaderPlugin;

pub struct UnitDataCollection {
    collection: std::collections::HashMap<String, UnitData>,
}

impl UnitDataCollection {
    
    pub fn new() -> Self {
        Self { collection: std::collections::HashMap::new() }
    }

    pub fn insert(&mut self, name: String, element: UnitData) {
        self.collection.insert(name, element);
    }

    pub fn get(&self, key: &String) -> Option<&UnitData> {
        self.collection.get(key)
    }

}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AssemblyData {
    name: String,
    platform: String,
    loadout: Vec<String>
}

#[derive(Default)]
pub struct AssemblyRegistry {
    assemblies: Vec<AssemblyData>,
}

impl AssemblyRegistry {
    pub fn new() -> Self {
        Self { assemblies: Vec::new() }
    }
    pub fn push(&mut self, item: AssemblyData) {
        self.assemblies.push(item);
    }
}

pub struct SubunitRegistry {
    collection: std::collections::HashMap<String, SubunitData>
}

impl SubunitRegistry {
    pub fn new() -> Self {
        Self { collection: std::collections::HashMap::new() }
    }
    pub fn insert(&mut self, name: String, element: SubunitData) {
        self.collection.insert(name, element);
    }
    pub fn get(&self, key: &String) -> Option<&SubunitData> {
        self.collection.get(key)
    }
}

pub struct PlatformRegistry {
    collection: std::collections::HashMap<String, PlatformData>
}

impl PlatformRegistry {
    pub fn new() -> Self {
        Self { collection: std::collections::HashMap::new() }
    }
    pub fn insert(&mut self, name: String, element: PlatformData) {
        self.collection.insert(name, element);
    }
    pub fn get(&self, key: &String) -> Option<&PlatformData> {
        self.collection.get(key)
    }
}

pub struct ProjectileRegistry {
    collection: std::collections::HashMap<String, ProjectileData>
}

impl ProjectileRegistry {
    pub fn new() -> Self {
        Self { collection: std::collections::HashMap::new() }
    }
    pub fn insert(&mut self, name: String, element: ProjectileData) {
        self.collection.insert(name, element);
    }
    pub fn get(&self, key: &String) -> Option<&ProjectileData> {
        self.collection.get(key)
    }
}

pub struct TextureServer {
    collection: std::collections::HashMap<String, HandleUntyped>
}

impl TextureServer {
    pub fn new() -> Self {
        Self { collection: std::collections::HashMap::new() }
    }
    pub fn insert(&mut self, name: String, element: HandleUntyped) {
        self.collection.insert(name, element);
    }
    pub fn get(&self, key: &String) -> HandleUntyped {
        if let Some(h) = self.collection.get(key) {
            return h.clone()
        }
        else {
            panic!("{} is not a valid texture.", key);
        }
    }
    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, std::string::String, bevy::prelude::HandleUntyped> {
        self.collection.keys()
    }
}

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource( AssemblyRegistry::new() )
            .insert_resource( SubunitRegistry::new() )
            .insert_resource( PlatformRegistry::new() )
            .insert_resource( ProjectileRegistry::new() )
            .insert_resource( UnitDataCollection::new() )
            .insert_resource( TextureServer::new() )
            .add_startup_system(load_textures_system)
            .add_startup_system(load_fonts_system)
            .add_startup_system(load_subunits_system.after(load_textures_system))
            .add_startup_system(load_projectiles_system.after(load_subunits_system))
            .add_startup_system(load_assemblies_system.after(load_projectiles_system))
            .add_startup_system(load_platforms_system.after(load_assemblies_system))
            .add_startup_system(create_unit_data_system.after(load_platforms_system));
            // .add_startup_system(load_platforms_system);
            // .add_system_set(SystemSet::new() // Input 
            //     .with_run_criteria(FixedTimestep::step(1. / 60.))  // VSYNC
            //     .with_system(inputs::input_mouse_system)
            //     .with_system(inputs::decode_action_system)
            // );
    }
}

pub struct Fonts {
    pub h2: Handle<Font>,
    pub serif_ui: Handle<Font>
}

const ASSEMBLY_DIR: &str = "assets/data/assemblies";
const SUBUNIT_DIR: &str = "assets/data/subunits";
const PLATFORM_DIR: &str = "assets/data/platforms";
const PROJECTILE_DIR: &str = "assets/data/projectiles";

fn load_fonts_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.insert_resource(Fonts {
        h2: asset_server.load("fonts/Oxanium-Medium.ttf"),
        serif_ui: asset_server.load("fonts/Elron-Monospace.ttf")
    });
    println!("Loaded fonts.")
}

fn load_textures_system(
    mut texture_server: ResMut<TextureServer>,
    mut atlas: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    'texture: for entry in glob::glob(&("**/*.png")).expect("Fatal: Invalid pattern") {
        match entry {
            Ok(path) => {
                if let Some(s) = path.to_str() {
                    let path_s = String::from(s).replace("\"", "");
                    let asset_path_s = path_s.replace("assets\\", "").replace("\\", "/");
                    println!("Loading texture from {}...", asset_path_s);
                    let texture_handle = asset_server.load(&asset_path_s);
                    // Check for JSON file describing the texture
                    let path_s_json = path_s.replace(".png", ".json");
                    if let Ok(json_s) = std::fs::read_to_string(&path_s_json) {
                        println!("   Loading texture data from {}", &path_s_json);
                        if let Ok(texture_data) = serde_json::from_str::<TextureData>(&json_s) {
                            let texture_atlas = TextureAtlas::from_grid(
                                texture_handle,
                                Vec2::new(texture_data.tile_size_x, texture_data.tile_size_y),
                                texture_data.columns,
                                texture_data.rows
                            );
                            texture_server.insert(asset_path_s.clone(), atlas.add(texture_atlas).clone_untyped());
                            continue 'texture   
                        }
                    }
                    texture_server.insert(asset_path_s.clone(), texture_handle.clone_untyped());
                }
                else {
                    eprintln!("Invalid texture path {:?}", path);
                }
            },
            Err(e) => eprintln!("{:?}", e)
        }
    }
}

fn load_assemblies_system(
    mut registry: ResMut<AssemblyRegistry>,
) {
    let assembly_paths = fs::read_dir(ASSEMBLY_DIR).unwrap();
    println!("Loading assemblies from {}", ASSEMBLY_DIR);
    for path in assembly_paths {
        if let Ok(path) = path {
            println!("    Found assembly {}.", path.path().display());
            if let Ok(s) = std::fs::read_to_string(path.path()) {
                match serde_json::from_str::<AssemblyData>(s.as_str()) {
                    Ok(assembly) => {
                        println!("       {:?}", assembly);
                        registry.push(assembly);
                    },
                    Err(e) => { eprintln!("Failed to parse {:?}, {:?}", s.as_str(), e) }
                }
            }
            else {
                println!("      Failed to load the data!");
            }
        }
    }
}

fn load_projectiles_system(
    mut registry: ResMut<ProjectileRegistry>,
) {
    for entry in glob::glob(&(PROJECTILE_DIR.to_owned() + "/**/*.json")).expect("Fatal: Invalid SUBUNIT_DIR") {
        match entry {
            Ok(path) => {
                if let Ok(s) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<ProjectileData>(s.as_str()) {
                        Ok(data) => {
                            println!("       {:?}", data.name);
                            registry.insert(data.name.clone(), data);
                        },
                        Err(e) => { eprintln!("Failed to parse {:?}, {:?}", s.as_str(), e) }
                    }
                }
            }  
            Err(e) => eprintln!("{:?}", e),
        };
    }
}

fn load_subunits_system(
    mut registry: ResMut<SubunitRegistry>,
) {
    for entry in glob::glob(&(SUBUNIT_DIR.to_owned() + "/**/*.json")).expect("Fatal: Invalid SUBUNIT_DIR") {
        match entry {
            Ok(path) => {
                if let Ok(s) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<SubunitData>(s.as_str()) {
                        Ok(data) => {
                            println!("       {:?}", data.name);
                            registry.insert(data.name.clone(), data);
                        },
                        Err(e) => { eprintln!("Failed to parse {:?}, {:?}", s.as_str(), e) }
                    }
                }  
            },
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

fn load_platforms_system(
    mut registry: ResMut<PlatformRegistry>,
) {
    for entry in glob::glob(&(PLATFORM_DIR.to_owned() + "/**/*.json")).expect("Fatal: Invalid SUBUNIT_DIR") {
        match entry {
            Ok(path) => {
                if let Ok(s) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<PlatformData>(s.as_str()) {
                        Ok(data) => {
                            println!("       {:?}", data.name);
                            registry.insert(data.name.clone(), data);
                        },
                        Err(e) => { eprintln!("Failed to parse {:?}, {:?}", s.as_str(), e) }
                    }
                }  
            },
            Err(e) => eprintln!("{:?}", e), 
        }
    }
}

fn create_unit_data_system(
    platform_registry: Res<PlatformRegistry>,
    subunit_registry: Res<SubunitRegistry>,
    assembly_registry: Res<AssemblyRegistry>,
    mut unit_data: ResMut<UnitDataCollection>,
) {
    println!("Loading Unit Data...");
    'assemblies: for assembly in assembly_registry.assemblies.iter() {
        println!("    Loading unit {}", assembly.name);
        println!("        Assembling on platform {}", assembly.platform);
        if let Some(platform) = platform_registry.get(&assembly.platform) {
            let mut loadout: Vec<SubunitData> = Vec::new();
            for (subunit_name, hardpoint) in zip(assembly.loadout.iter(), platform.hardpoints.iter()) {
                println!("        Assembling from subunit {}", subunit_name);
                if let Some(subunit) = subunit_registry.get(subunit_name) {
                    // Verify that the hardpoint fits the subunit
                    if subunit.hardpoint_size == hardpoint.hardpoint_size {
                        loadout.push(subunit.clone());
                    }
                    else {
                        eprintln!("        ...Loading failed. Invalid hardpoint '{}'", subunit_name);
                        continue 'assemblies
                    }
                }
                else {
                    eprintln!("        ...Loading failed. Could not resolve subunit '{}'", subunit_name);
                    continue 'assemblies  // Give up on loading this Unit
                }
            }
            unit_data.insert(
                assembly.name.clone(),
                UnitData {
                    name: assembly.name.clone(),
                    platform: platform.clone(),
                    loadout: loadout,
                }
            )
        }
        else {
            eprintln!("        ...Loading failed. Could not resolve platform '{}'", assembly.platform);
        }
    }
}

pub fn load_projectile(path: &str) -> Option<ProjectileData> {
    if let Ok(s) = std::fs::read_to_string(&path) {
        let data: ProjectileData = serde_json::from_str(s.as_str()).unwrap_or_else(|err| {
            panic!("Could not deserialize projectile {}: {:?}", path, err);
        });
        return Some(data)
    }  
    panic!("Could not read file {}", path);
}
