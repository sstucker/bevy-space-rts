use serde;
use serde_json;
use serde::{Deserialize, Serialize};
use glob;

use std::{fs, collections::HashMap};

use bevy::prelude::*;

use crate::*;

pub struct AssetLoaderPlugin;

pub struct UnitDataElement {
    name: String,
    platform: PlatformData,
    loadout: Vec<SubunitData>
}


pub struct UnitDataCollection {
    collection: std::collections::HashMap<String, UnitDataElement>,
}

impl UnitDataCollection {
    
    pub fn new() -> Self {
        Self { collection: std::collections::HashMap::new() }
    }

    pub fn insert(&mut self, name: String, element: UnitDataElement) {
        self.collection.insert(name, element);
    }

    pub fn get(&self, key: String) -> &UnitDataElement {
        self.collection.get(&key).unwrap()
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
    pub fn get(&self, key: String) -> &SubunitData {
        self.collection.get(&key).unwrap()
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
pub fn get(&self, key: String) -> &PlatformData {
    self.collection.get(&key).unwrap()
}
}



impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource( AssemblyRegistry::new() )
            .insert_resource( SubunitRegistry::new() )
            .insert_resource( PlatformRegistry::new() )
            .insert_resource( UnitDataCollection::new() )
            .add_startup_system(load_assemblies_system)
            .add_startup_system(load_subunits_system.after(load_assemblies_system))
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
                    println!("       Imported Subunit {:?}", data["name"].as_str().unwrap());
                    registry.insert(data["name"].to_string(), data);
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
                    println!("       Imported Platform {:?}", data["name"].as_str().unwrap());
                    registry.insert(data["name"].to_string(), data);
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
    unit_data: ResMut<UnitDataCollection>,
) {
    println!("Loading Unit Data...");
    for assembly in assembly_registry.assemblies.iter() {
        println!("    Loading unit {}", assembly.name);
        println!("        Assembling on platform {}", assembly.platform);
        for subunit_name in assembly.loadout.iter() {
            println!("        Assembling from subunit {}", subunit_name);
        }
    }
}

