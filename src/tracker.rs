use std::collections::HashMap;
use rust_decimal::prelude::*;

use crate::{session::{Session, SessionSkill, SessionLoot}, loadout::Loadout, logger::{Log, EventType}};

pub trait Base {
    fn track(&mut self, log: Log) -> &Tracker;
}

pub struct Tracker {
    pub current_session: Session,
    pub loudouts: HashMap<String, Loadout>,
    pub sessions: HashMap<String, Session>,
}

impl Tracker {
    pub fn new() -> Tracker {
        return Tracker {
            current_session: Session::new(),
            loudouts: HashMap::new(),
            sessions: HashMap::new(),
        };
    }
}

impl Base for Tracker {
    fn track(&mut self, log: Log) -> &Tracker {
        match log.event_type {
            EventType::SelfCrit => {
                self.current_session.stats.self_crit_count += 1;
                self.current_session.stats.self_total_damage += Decimal::from_str(log.values.first().unwrap()).unwrap();
                self.current_session.stats.self_total_crit_damage += Decimal::from_str(log.values.first().unwrap()).unwrap();
            },
            EventType::SelfHit => {
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_total_damage += Decimal::from_str(log.values.first().unwrap()).unwrap();
            },
            EventType::SelfHeal => {
                self.current_session.stats.self_total_heal += Decimal::from_str(log.values.first().unwrap()).unwrap();
            },
            EventType::SelfDeflect => {
                self.current_session.stats.self_deflect_count += 1;
                self.current_session.stats.target_attack_count += 1;
            },
            EventType::SelfEvade => {
                self.current_session.stats.self_evade_count += 1;
                self.current_session.stats.target_attack_count += 1;
            },
            EventType::SelfMiss => {
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_attack_miss_count += 1;
            },
            EventType::SelfSkillGain => {
                let exp_gain = Decimal::from_str(log.values.first().unwrap()).unwrap();
                let skill = log.values.get(1).unwrap();
                self.current_session.stats.self_total_exp_gain += exp_gain;
                if self.current_session.skill_map.contains_key(skill) {
                    self.current_session.skill_map.get_mut(skill).unwrap().exp_gain += exp_gain;
                } else {
                    self.current_session.skill_map.insert(skill.to_string(), SessionSkill {
                        name: skill.to_string(),
                        exp_gain,
                    });
                }
            },
            EventType::SelfLoot => {
                let loot = log.values.first().unwrap();
                let quantity = log.values.get(1).unwrap().parse::<usize>().unwrap();
                let value = Decimal::from_str(log.values.get(2).unwrap()).unwrap();
                if self.current_session.loot_map.contains_key(loot) {
                    self.current_session.loot_map.get_mut(loot).unwrap().count += quantity;
                } else {
                    self.current_session.loot_map.insert(loot.to_string(), SessionLoot {
                        name: loot.to_string(),
                        tt_value: value,
                        // TODO: Add MU
                        mu_value: value,
                        count: quantity,
                    });
                }
            },
            EventType::TargetDodge => {
                self.current_session.stats.target_dodge_count += 1;
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_attack_miss_count += 1;
            },
            EventType::TargetEvade => {
                self.current_session.stats.target_evade_count += 1;
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_attack_miss_count += 1;
            },
            EventType::TargetJam => {
                self.current_session.stats.target_jam_count += 1;
                self.current_session.stats.self_attack_count += 1;
                self.current_session.stats.self_attack_miss_count += 1;
            },
            EventType::TargetHit => {
                self.current_session.stats.target_attack_count += 1;
                self.current_session.stats.target_total_damage += Decimal::from_str(log.values.first().unwrap()).unwrap();
            },
            EventType::GlobalHuntHOF => {
                let global_value = Decimal::from_str(log.values.get(2).unwrap()).unwrap();
                self.current_session.stats.global_count += 1;
                self.current_session.stats.hof_count += 1;
                self.current_session.stats.total_global_gain += global_value;
                self.current_session.stats.total_hof_gain += global_value;
            },
            EventType::GlobalHunt => {
                let global_value = Decimal::from_str(log.values.get(2).unwrap()).unwrap();
                self.current_session.stats.global_count += 1;
                self.current_session.stats.total_global_gain += global_value;
            },
        }

        return self;
    }
}
