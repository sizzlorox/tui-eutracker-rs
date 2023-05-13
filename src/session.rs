use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Duration, Utc};
use serde_with::{serde_as, DurationSeconds};

use glob::glob;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Write, path::Path};

use crate::loadout::Loadout;

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub name: String,
    #[serde(with = "ts_seconds_option")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde_as(as = "DurationSeconds<i64>")]
    pub elapsed_time: Duration,
    pub is_active: bool,

    pub loadout: Loadout,
    pub stats: SessionStats,
    pub loot_map: HashMap<String, SessionLoot>,
    pub skill_map: HashMap<String, SessionSkill>,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(session_name: &str) -> Session {
        let current_session_file = Path::new(session_name);
        let default_session = Session {
            name: String::from(session_name.replace(".json", "")),
            start_time: None,
            elapsed_time: Duration::zero(),
            is_active: false,
            loadout: Loadout::new("default"),
            stats: SessionStats::new(),
            loot_map: HashMap::new(),
            skill_map: HashMap::new(),
            created_at: Utc::now(),
        };

        let mut file = File::create(current_session_file).unwrap();
        let contents = serde_json::to_string_pretty(&default_session).unwrap();
        file.write_all(contents.as_bytes()).unwrap();

        default_session
    }

    pub fn load(path: &Path) -> Option<Session> {
        if !path.exists() {
            return None;
        }
        let contents = std::fs::read_to_string(path).unwrap();
        let session: Session = serde_json::from_str(&contents).unwrap();

        Some(session)
    }

    pub fn fetch() -> HashMap<String, Session> {
        let mut session_map: HashMap<String, Session> = HashMap::new();
        for entry in glob("*_session.json").unwrap() {
            if let Ok(file_path) = entry {
                if file_path.to_str().unwrap() == "current_session.json" {
                    continue;
                }
                let session = Session::load(&file_path).unwrap();
                session_map.insert(
                    file_path.file_stem().unwrap().to_str().unwrap().to_string(),
                    session,
                );
            }
        }

        session_map
    }

    pub fn save(self: &Self) {
        let file_name = format!("{}.json", &self.name);
        let current_session_file = Path::new(file_name.as_str());
        let mut file = File::create(current_session_file).unwrap();
        let contents = serde_json::to_string_pretty(self).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionStats {
    pub tt_profit: Decimal,
    pub total_cost: Decimal,
    pub global_count: usize,
    pub total_global_gain: Decimal,
    pub hof_count: usize,
    pub total_hof_gain: Decimal,

    pub self_total_exp_gain: Decimal,

    pub self_total_crit_damage: Decimal,
    pub self_total_damage: Decimal,
    pub self_total_heal: Decimal,
    pub self_attack_miss_count: usize,
    pub self_attack_count: usize,
    pub self_crit_count: usize,
    pub self_evade_count: usize,
    pub self_deflect_count: usize,
    pub self_death_count: usize,

    pub target_total_damage: Decimal,
    pub target_attack_count: usize,
    pub target_dodge_count: usize,
    pub target_evade_count: usize,
    pub target_jam_count: usize,
}

impl SessionStats {
    pub fn new() -> SessionStats {
        return SessionStats {
            tt_profit: Decimal::new(0, 6),
            total_cost: Decimal::new(0, 6),
            global_count: 0,
            total_global_gain: Decimal::new(0, 6),
            hof_count: 0,
            total_hof_gain: Decimal::new(0, 6),
            self_total_exp_gain: Decimal::new(0, 6),
            self_total_crit_damage: Decimal::new(0, 6),
            self_total_damage: Decimal::new(0, 6),
            self_total_heal: Decimal::new(0, 6),
            self_attack_count: 0,
            self_attack_miss_count: 0,
            self_crit_count: 0,
            self_evade_count: 0,
            self_deflect_count: 0,
            self_death_count: 0,
            target_total_damage: Decimal::new(0, 6),
            target_attack_count: 0,
            target_dodge_count: 0,
            target_evade_count: 0,
            target_jam_count: 0,
        };
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionLoot {
    pub name: String,
    pub tt_value: Decimal,
    pub count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionSkill {
    pub name: String,
    pub exp_gain: Decimal,
}

pub trait Stopwatch {
    fn start(&mut self);
    fn pause(&mut self);
    fn reset(&mut self);
    fn elapsed(&self) -> Duration;
    fn pretty_elapsed(&self) -> String;
}

impl Stopwatch for Session {
    fn start(&mut self) {
        self.start_time = Some(Utc::now());
        self.is_active = true;
    }
    fn pause(&mut self) {
        self.elapsed_time =
            self.elapsed_time + Utc::now().signed_duration_since(self.start_time.unwrap());
        self.start_time = None;
        self.is_active = false;
    }
    fn reset(&mut self) {
        self.start_time = Some(Utc::now());
        self.elapsed_time = Duration::seconds(0);
        self.is_active = false;
    }
    fn elapsed(&self) -> Duration {
        if self.is_active {
            return self.elapsed_time + Utc::now().signed_duration_since(self.start_time.unwrap());
        }
        return self.elapsed_time;
    }
    fn pretty_elapsed(&self) -> String {
        let elapsed = self.elapsed();
        let hours = elapsed.num_seconds() / 3600;
        let minutes = (elapsed.num_seconds() % 3600) / 60;
        let seconds = elapsed.num_seconds() % 60;
        let millis = elapsed.num_milliseconds();

        format!(
            "{:02}h {:02}m {:02}s {:03}ms",
            hours, minutes, seconds, millis
        )
    }
}
