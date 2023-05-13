use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use std::{collections::HashMap, fs::File, io::Write, path::Path};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Clone)]
pub struct Markup {
    pub name: String,
    pub value: Decimal,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Markup {
    pub fn load() -> HashMap<String, Markup> {
        let markup_file = Path::new("markups.json");
        if !markup_file.exists() {
            let default_markups: HashMap<String, Markup> = HashMap::new();
            let mut file = File::create(markup_file).unwrap();
            let contents = serde_json::to_string_pretty(&json!({})).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
            return default_markups;
        }
        let contents = std::fs::read_to_string(markup_file).unwrap();
        let markups: HashMap<String, Markup> = serde_json::from_str(&contents).unwrap();

        markups
    }

    pub fn save(markups: HashMap<String, Markup>) {
        let markup_file = Path::new("markups.json");
        let mut file = File::create(markup_file).unwrap();
        let contents = serde_json::to_string_pretty(&markups).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }
}
