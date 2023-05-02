use std::{time::{Instant, Duration}, collections::HashMap};

use rust_decimal::Decimal;

use crate::loadout::Loadout;

pub struct Session {
    pub start_time: Instant,
    pub elapsed_time: Duration,
    pub is_active: bool,

    pub loadout: Loadout,
    pub stats: SessionStats,
    pub loot_map: HashMap<String, SessionLoot>,
    pub skill_map: HashMap<String, SessionSkill>,
}

impl Session {
    pub fn new() -> Session {
        return Session {
            start_time: Instant::now(),
            elapsed_time: Duration::from_secs(0),
            is_active: false,
            loadout: Loadout::new(),
            stats: SessionStats::new(),
            loot_map: HashMap::new(),
            skill_map: HashMap::new(),
        };
    }
}

pub struct SessionStats {
    pub mu_profit: Decimal,
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

    pub target_total_damage: Decimal,
    pub target_attack_count: usize,
    pub target_dodge_count: usize,
    pub target_evade_count: usize,
    pub target_jam_count: usize,
}

impl SessionStats {
    pub fn new() -> SessionStats {
        return SessionStats {
            mu_profit: Decimal::new(0, 6),
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
            target_total_damage: Decimal::new(0, 6),
            target_attack_count: 0,
            target_dodge_count: 0,
            target_evade_count: 0,
            target_jam_count: 0,
        };
    }
}

pub struct SessionLoot {
    pub name: String,
    pub tt_value: Decimal,
    pub mu_value: Decimal,
    pub count: usize,
}

pub struct SessionSkill {
    pub name: String,
    pub exp_gain: Decimal,
}
