use chrono::Local;
use rust_decimal::prelude::*;
use std::{
    collections::{HashMap, VecDeque},
    path::Path,
};

use crate::{
    loadout::Loadout,
    logger::{EventType, Log},
    session::{Session, SessionLoot, SessionSkill},
};

pub trait Base {
    fn track(&mut self, log: Log) -> &Tracker;
}

pub struct Tracker {
    pub user: String,

    pub current_session: Session,
    pub loadouts: HashMap<String, Loadout>,
    pub sessions: HashMap<String, Session>,
    pub logs: VecDeque<String>,
}

impl Tracker {
    pub fn new(user: String) -> Tracker {
        let sessions = Session::fetch();
        let mut sessions_vec: Vec<&Session> = sessions.values().into_iter().collect();
        sessions_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let default_session = sessions_vec.get(0);
        if default_session.is_some() {
            let default_load_session_file = format!("{}.json", sessions_vec.get(0).unwrap().name);
            match Session::load(Path::new(default_load_session_file.as_str())) {
                Some(session) => {
                    return Tracker {
                        user,
                        current_session: session,
                        loadouts: Loadout::fetch(),
                        sessions,
                        logs: VecDeque::with_capacity(75),
                    }
                }
                None => {
                    let date_string = Local::now().format("%Y-%m-%d_%H-%M-%S");
                    return Tracker {
                        user,
                        current_session: Session::new(
                            format!("{}_session.json", date_string).as_str(),
                        ),
                        loadouts: Loadout::fetch(),
                        sessions: Session::fetch(),
                        logs: VecDeque::with_capacity(75),
                    };
                }
            }
        }
        let date_string = Local::now().format("%Y-%m-%d_%H-%M-%S");
        return Tracker {
            user,
            current_session: Session::new(format!("{}_session.json", date_string).as_str()),
            loadouts: Loadout::fetch(),
            sessions: Session::fetch(),
            logs: VecDeque::with_capacity(75),
        };
    }
}

impl Base for Tracker {
    fn track(&mut self, log: Log) -> &Tracker {
        let mut push_to_logs = true;
        match log.event_type {
            EventType::SelfCrit => {
                self.current_session.stats.self_crit_count += 1;
                self.current_session.stats.self_total_damage +=
                    Decimal::from_str(log.values.get(0).unwrap()).unwrap();
                self.current_session.stats.self_total_crit_damage +=
                    Decimal::from_str(log.values.get(0).unwrap()).unwrap();
            }
            EventType::SelfHit => {
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_total_damage +=
                    Decimal::from_str(log.values.get(0).unwrap()).unwrap();
                // self.current_session.stats.total_cost +=
            }
            EventType::SelfHeal => {
                self.current_session.stats.self_total_heal +=
                    Decimal::from_str(log.values.get(0).unwrap()).unwrap();
            }
            EventType::SelfDeflect => {
                self.current_session.stats.self_deflect_count += 1;
                self.current_session.stats.target_attack_count += 1;
            }
            EventType::SelfEvade => {
                self.current_session.stats.self_evade_count += 1;
                self.current_session.stats.target_attack_count += 1;
            }
            EventType::SelfMiss => {
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_attack_miss_count += 1;
            }
            EventType::SelfSkillGain => {
                let exp_gain = Decimal::from_str(log.values.get(0).unwrap()).unwrap();
                let skill = log.values.get(1).unwrap();
                self.current_session.stats.self_total_exp_gain += exp_gain;
                if self.current_session.skill_map.contains_key(skill) {
                    self.current_session
                        .skill_map
                        .get_mut(skill)
                        .unwrap()
                        .exp_gain += exp_gain;
                } else {
                    self.current_session.skill_map.insert(
                        skill.to_string(),
                        SessionSkill {
                            name: skill.to_string(),
                            exp_gain,
                        },
                    );
                }
            }
            EventType::SelfLoot => {
                let loot = log.values.get(0).unwrap();
                let quantity = log.values.get(1).unwrap().parse::<usize>().unwrap();
                let value = Decimal::from_str(log.values.get(2).unwrap()).unwrap();
                self.current_session.stats.mu_profit += value;
                self.current_session.stats.tt_profit += value;
                if self.current_session.loot_map.contains_key(loot) {
                    self.current_session.loot_map.get_mut(loot).unwrap().count += quantity;
                } else {
                    self.current_session.loot_map.insert(
                        loot.to_string(),
                        SessionLoot {
                            name: loot.to_string(),
                            tt_value: value,
                            // TODO: Add MU
                            mu_value: value,
                            count: quantity,
                        },
                    );
                }
            }
            EventType::TargetDodge => {
                self.current_session.stats.target_dodge_count += 1;
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_attack_miss_count += 1;
            }
            EventType::TargetEvade => {
                self.current_session.stats.target_evade_count += 1;
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_attack_miss_count += 1;
            }
            EventType::TargetJam => {
                self.current_session.stats.target_jam_count += 1;
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_attack_miss_count += 1;
            }
            EventType::TargetHit => {
                self.current_session.stats.target_attack_count += 1;
                self.current_session.stats.target_total_damage +=
                    Decimal::from_str(log.values.get(0).unwrap()).unwrap();
            }
            EventType::GlobalHuntHOF => {
                let global_user: String = log.values.get(0).unwrap().to_string();
                if global_user == self.user {
                    let global_value = Decimal::from_str(log.values.get(2).unwrap()).unwrap();
                    self.current_session.stats.global_count += 1;
                    self.current_session.stats.hof_count += 1;
                    self.current_session.stats.total_global_gain += global_value;
                    self.current_session.stats.total_hof_gain += global_value;
                } else {
                    push_to_logs = false;
                }
            }
            EventType::GlobalHunt => {
                let global_user: String = log.values.get(0).unwrap().to_string();
                if global_user == self.user {
                    let global_value = Decimal::from_str(log.values.get(2).unwrap()).unwrap();
                    self.current_session.stats.global_count += 1;
                    self.current_session.stats.total_global_gain += global_value;
                } else {
                    push_to_logs = false;
                }
            }
        }
        if push_to_logs {
            self.logs.push_front(log.line.to_string());
        }
        self.logs.truncate(75);

        return self;
    }
}
