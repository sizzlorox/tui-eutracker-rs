use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum LogType {
    Combat,
    Global,
    Loot,
    Skills,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum EventType {
    SelfCrit,
    SelfHit,
    SelfHeal,
    SelfDeflect,
    SelfEvade,
    SelfMiss,
    SelfSkillGain,
    SelfLoot,
    TargetDodge,
    TargetEvade,
    TargetJam,
    TargetHit,
    GlobalHuntHOF,
    GlobalHunt,
}

pub struct Log<'a> {
    pub line: &'a str,
    pub log_type: LogType,
    pub event_type: EventType,
    pub values: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct LogEvent<'a> {
    pub regex: &'a str,
    pub log_type: LogType,
    pub event_type: EventType,
}

pub struct Logger<'a> {
    pub log_events: HashMap<usize, LogEvent<'a>>,
}

impl<'a> Logger<'a> {
    pub fn new() -> Logger<'a> {
        let mut log_events = HashMap::new();
        log_events.insert(0, LogEvent {
            regex: r"Critical hit - Additional damage! You inflicted (.*?) points of damage",
            log_type: LogType::Combat,
            event_type: EventType::SelfCrit,
        });
        log_events.insert(1, LogEvent {
            regex: r"You inflicted (.*?) points of damage",
            log_type: LogType::Combat,
            event_type: EventType::SelfHit,
        });
        log_events.insert(2, LogEvent {
            regex: r"You healed yourself (.*?) points",
            log_type: LogType::Combat,
            event_type: EventType::SelfHeal,
        });
        log_events.insert(3, LogEvent {
            regex: r"Damage deflected!",
            log_type: LogType::Combat,
            event_type: EventType::SelfDeflect,
        });
        log_events.insert(4, LogEvent {
            regex: r"You Evaded the attack",
            log_type: LogType::Combat,
            event_type: EventType::SelfEvade,
        });
        log_events.insert(5, LogEvent {
            regex: r"You missed",
            log_type: LogType::Combat,
            event_type: EventType::SelfMiss,
        });
        log_events.insert(6, LogEvent {
            regex: r"You have gained (.*?) experience in your (.*?) skill",
            log_type: LogType::Skills,
            event_type: EventType::SelfSkillGain,
        });
        log_events.insert(7, LogEvent {
            regex: r"You received (.*?) x \((.*?)\) (.*?) PED",
            log_type: LogType::Loot,
            event_type: EventType::SelfLoot,
        });
        log_events.insert(8, LogEvent {
            regex: r"The target Dodged your attack",
            log_type: LogType::Combat,
            event_type: EventType::TargetDodge,
        });
        log_events.insert(9, LogEvent {
            regex: r"The target Evaded your attack",
            log_type: LogType::Combat,
            event_type: EventType::TargetEvade,
        });
        log_events.insert(10, LogEvent {
            regex: r"The target Jammed your attack",
            log_type: LogType::Combat,
            event_type: EventType::TargetJam,
        });
        log_events.insert(11, LogEvent {
            regex: r"You took (.*?) points of damage",
            log_type: LogType::Combat,
            event_type: EventType::TargetHit,
        });
        log_events.insert(12, LogEvent {
            regex: r"\[\] (.*?) killed a creature \((.*?)\) with a value of (.*?) PED! A record has been added to the Hall of Fame!",
            log_type: LogType::Global,
            event_type: EventType::GlobalHuntHOF,
        });
        log_events.insert(13, LogEvent {
            regex: r"\[\] (.*?) killed a creature \((.*?)\) with a value of (.*?) PED!",
            log_type: LogType::Global,
            event_type: EventType::GlobalHunt,
        });

        return Logger { log_events };
    }
}