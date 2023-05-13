use chrono::Local;
use chrono::Utc;
use rust_decimal::prelude::*;
use std::{
    collections::{HashMap, VecDeque},
    ops::Deref,
    path::Path,
};

use crate::{
    loadout::Loadout,
    logger::{EventType, Log},
    markup::Markup,
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
    pub markups: HashMap<String, Markup>,
    pub logs: VecDeque<String>,
}

impl Tracker {
    pub fn new(user: String) -> Tracker {
        let sessions = Session::fetch();
        let mut sessions_vec: Vec<&Session> = sessions.values().into_iter().collect();
        sessions_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let default_session = sessions_vec.get(0);

        let loadouts = Loadout::fetch();

        if default_session.is_some() {
            let default_load_session_file = format!("{}.json", sessions_vec.get(0).unwrap().name);

            match Session::load(Path::new(default_load_session_file.as_str())) {
                Some(mut session) => {
                    let mut loadouts_vec: Vec<&Loadout> = loadouts.values().into_iter().collect();
                    loadouts_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                    let active_loadout_idx = loadouts_vec
                        .iter()
                        .position(|&s| s.name == session.loadout.name);

                    session.loadout = loadouts_vec[active_loadout_idx.unwrap()].deref().clone();
                    session.is_active = false;

                    return Tracker {
                        user,
                        current_session: session,
                        loadouts,
                        sessions,
                        markups: Markup::load(),
                        logs: VecDeque::with_capacity(75),
                    };
                }
                None => {
                    let date_string = Local::now().format("%Y-%m-%d_%H-%M-%S");
                    return Tracker {
                        user,
                        current_session: Session::new(
                            format!("{}_session.json", date_string).as_str(),
                        ),
                        loadouts,
                        sessions: Session::fetch(),
                        markups: Markup::load(),
                        logs: VecDeque::with_capacity(75),
                    };
                }
            }
        }
        let date_string = Local::now().format("%Y-%m-%d_%H-%M-%S");
        return Tracker {
            user,
            current_session: Session::new(format!("{}_session.json", date_string).as_str()),
            loadouts,
            sessions: Session::fetch(),
            markups: Markup::load(),
            logs: VecDeque::with_capacity(75),
        };
    }
}

impl Base for Tracker {
    fn track(&mut self, log: Log) -> &Tracker {
        if !self.current_session.is_active {
            return self;
        }

        let mut push_to_logs = true;
        match log.event_type {
            EventType::SelfCrit => {
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_crit_count += 1;
                self.current_session.stats.self_total_damage +=
                    Decimal::from_str_exact(log.values.get(0).unwrap()).unwrap();
                self.current_session.stats.self_total_crit_damage +=
                    Decimal::from_str_exact(log.values.get(0).unwrap()).unwrap();
                self.current_session.stats.total_cost +=
                    Decimal::from(self.current_session.loadout.burn)
                        .checked_div(
                            Decimal::from(10000)
                                + self.current_session.loadout.decay * Decimal::new(1, 2),
                        )
                        .unwrap_or(Decimal::ZERO);
            }
            EventType::SelfHit => {
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_total_damage +=
                    Decimal::from_str_exact(log.values.get(0).unwrap()).unwrap();
                self.current_session.stats.total_cost +=
                    Decimal::from(self.current_session.loadout.burn)
                        .checked_div(
                            Decimal::from(10000)
                                + self.current_session.loadout.decay * Decimal::new(1, 2),
                        )
                        .unwrap_or(Decimal::ZERO);
            }
            EventType::SelfHeal => {
                self.current_session.stats.self_total_heal +=
                    Decimal::from_str_exact(log.values.get(0).unwrap()).unwrap();
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
                let exp_gain = Decimal::from_str_exact(log.values.get(0).unwrap()).unwrap();
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
                let value = Decimal::from_str_exact(log.values.get(2).unwrap()).unwrap();
                self.current_session.stats.tt_profit += value;

                if !self.markups.contains_key(loot) {
                    self.markups.insert(
                        loot.to_string(),
                        Markup {
                            name: loot.to_string(),
                            value: Decimal::new(100, 2),
                            created_at: Utc::now(),
                        },
                    );
                }

                if self.current_session.loot_map.contains_key(loot) {
                    self.current_session
                        .loot_map
                        .get_mut(loot)
                        .unwrap()
                        .tt_value += value;
                    self.current_session.loot_map.get_mut(loot).unwrap().count += quantity;
                } else {
                    self.current_session.loot_map.insert(
                        loot.to_string(),
                        SessionLoot {
                            name: loot.to_string(),
                            tt_value: value,
                            count: quantity,
                        },
                    );
                }
            }
            EventType::SelfDeath => {
                self.current_session.stats.self_death_count += 1;
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
                    Decimal::from_str_exact(log.values.get(0).unwrap()).unwrap();
            }
            EventType::GlobalHuntHOF => {
                let global_user: String = log.values.get(0).unwrap().to_string();
                if global_user == self.user {
                    let global_value = Decimal::from_str_exact(log.values.get(2).unwrap()).unwrap();
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
                    let global_value = Decimal::from_str_exact(log.values.get(2).unwrap()).unwrap();
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
