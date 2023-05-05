use glob::glob;
use std::{collections::HashMap, fs::File, io::Write, path::Path};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Loadout {
    pub name: String,
    pub weapon: Option<String>,
    pub amp: Option<String>,
    pub scope: Option<String>,
    pub sight_one: Option<String>,
    pub sight_two: Option<String>,
    pub decay: Decimal,
    pub burn: usize,
}

impl Loadout {
    pub fn new(loadout_name: &str) -> Loadout {
        let formatted_name = format!(
            "{}_loadout.json",
            loadout_name.replace(" ", "_").to_lowercase()
        );
        let current_loadout_file = Path::new(formatted_name.as_str());
        let default_loadout = Loadout {
            name: String::from(loadout_name),
            weapon: None,
            amp: None,
            scope: None,
            sight_one: None,
            sight_two: None,
            decay: Decimal::new(0, 6),
            burn: 0,
        };

        let mut file = File::create(current_loadout_file).unwrap();
        let contents = serde_json::to_string_pretty(&default_loadout).unwrap();
        file.write_all(contents.as_bytes()).unwrap();

        default_loadout
    }

    pub fn load(path: &Path) -> Option<Loadout> {
        if !path.exists() {
            return None;
        }
        let contents = std::fs::read_to_string(path).unwrap();
        let loadout: Loadout = serde_json::from_str(&contents).unwrap();

        Some(loadout)
    }

    pub fn fetch() -> HashMap<String, Loadout> {
        let mut loadout_map: HashMap<String, Loadout> = HashMap::new();
        for entry in glob("*_loadout.json").unwrap() {
            if let Ok(file_path) = entry {
                let loadout = Loadout::load(&file_path).unwrap();
                loadout_map.insert(
                    file_path.file_stem().unwrap().to_str().unwrap().to_string(),
                    loadout,
                );
            }
        }

        loadout_map
    }

    pub fn save(self: &Self) {
        let formatted_name = format!(
            "{}_loadout.json",
            self.name.replace(" ", "_").to_lowercase()
        );
        let current_loadout_file = Path::new(formatted_name.as_str());
        let mut file = File::create(current_loadout_file).unwrap();
        let contents = serde_json::to_string_pretty(self).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    pub fn export(self: &Self, loadout_name: &str) {
        let current_loadout_file = Path::new(loadout_name);
        let mut file: File = File::create(current_loadout_file).unwrap();
        let contents = serde_json::to_string_pretty(self).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }
}
