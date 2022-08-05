use serde;
use serde_json;
use serde::{Deserialize, Serialize};
use glob;

use std::{fs, collections::HashMap, iter::zip};

use bevy::prelude::*;

use crate::*;

pub struct AssetLoaderPlugin;

pub struct UnitData {
    pub name: String,
    pub platform: PlatformData,
    pub loadout: Vec<SubunitData>
}

impl UnitData {
    pub fn new(name: String, platform_data: PlatformData, subunit_data: Vec<SubunitData>) -> Self {
        Self {
            name: name,
            platform: platform_data,
            loadout: subunit_data
        }
    }
}


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

// TODO make these strongly typed?
type SubunitData = serde_json::Value;
type PlatformData = serde_json::Value;
type SpriteData = serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct ProjectileData {
    pub name: String,
    pub class: String,
    pub subclass: String,
    pub velocity: f32,
    pub range: f32,
    pub size: Vec<f32>,
    pub sprites: Vec<SpriteData>
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

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource( AssemblyRegistry::new() )
            .insert_resource( SubunitRegistry::new() )
            .insert_resource( PlatformRegistry::new() )
            .insert_resource( ProjectileRegistry::new() )
            .insert_resource( UnitDataCollection::new() )
            .add_startup_system(load_assemblies_system)
            .add_startup_system(load_subunits_system.after(load_assemblies_system))
            .add_startup_system(load_projectiles_system.after(load_subunits_system))
            .add_startup_system(load_platforms_system.after(load_subunits_system))
            .add_startup_system(create_unit_data_system.after(load_platforms_system));
            // .add_startup_system(load_platforms_system);
            // .add_system_set(SystemSet::new() // Input 
            //     .with_run_criteria(FixedTimestep::step(1. / 60.))  // VSYNC
            //     .with_system(inputs::input_mouse_system)
            //     .with_system(inputs::decode_action_system)
            // );
    }
}

const ASSEMBLY_DIR: &str = "assets/data/assemblies";
const SUBUNIT_DIR: &str = "assets/data/subunits";
const PLATFORM_DIR: &str = "assets/data/platforms";
const PROJECTILE_DIR: &str = "assets/data/projectiles";

fn load_assemblies_system(
    mut registry: ResMut<AssemblyRegistry>,
) {
    let assembly_paths = fs::read_dir(ASSEMBLY_DIR).unwrap();
    println!("Loading assemblies from {}", ASSEMBLY_DIR);
    for path in assembly_paths {
        if let Ok(path) = path {
            println!("    Found assembly {}.", path.path().display());
            if let Ok(s) = std::fs::read_to_string(path.path()) {
                let assembly: AssemblyData = serde_json::from_str(s.as_str()).unwrap();
                println!("       {:?}", assembly);
                registry.push(assembly);
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
                    let data: ProjectileData = serde_json::from_str(s.as_str()).unwrap_or_else(|err| {
                        panic!("Could not deserialize {}: {:?}", path.display(), err);
                    });
                    let projectile_name = String::from(&data.name);
                    println!("       Imported Projectile {:?}", projectile_name);
                    registry.insert(projectile_name, data);
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
                    let data: SubunitData = serde_json::from_str(s.as_str()).unwrap_or_else(|err| {
                        panic!("Could not deserialize {}: {:?}", path.display(), err);
                    });
                    if let Some(v) = data.get("name") {
                        if let Some(vs) = v.as_str() {
                            let subunit_name = String::from(vs);
                            println!("       Imported Subunit {:?}", subunit_name);
                            registry.insert(subunit_name, data);
                        }
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
                    let data: PlatformData = serde_json::from_str(s.as_str()).unwrap_or_else(|err| {
                        panic!("Could not deserialize {}: {:?}", path.display(), err);
                    });
                    if let Some(platform_name) = data.get("name") {
                        println!("       Imported Platform {:?}", platform_name.to_string().replace("\"", ""));
                        registry.insert(platform_name.to_string().replace("\"", ""), data);
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
        if let Some(platform) = platform_registry.get(&assembly.platform).clone(){
            let mut loadout: Vec<SubunitData> = Vec::new();
            for (subunit_name, hardpoint) in zip(assembly.loadout.iter(), platform["hardpoints"].as_array().unwrap()) {
                println!("        Assembling from subunit {}", subunit_name);
                if let Some(subunit) = subunit_registry.get(subunit_name) {
                    // Verify that the hardpoint fits the subunit
                    if subunit["class"].as_str().unwrap() == hardpoint["class"].as_str().unwrap()
                    && subunit["hardpoint-class"].as_i64().unwrap() == hardpoint["hardpoint-class"].as_i64().unwrap() {
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
            panic!("Could not deserialize {}: {:?}", path, err);
        });
        return Some(data)
    }  
    panic!("Could not read file {}", path);
}
