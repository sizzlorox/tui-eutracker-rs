use rust_decimal::Decimal;

pub struct Loadout {
    pub name: String,
    pub amp: Option<String>,
    pub scope: Option<String>,
    pub sight_one: Option<String>,
    pub sight_two: Option<String>,
    pub decay: Decimal,
    pub burn: usize,
}

impl Loadout {
    pub fn new() -> Loadout {
        return Loadout {
            name: String::from("Default Loadout"),
            amp: None,
            scope: None,
            sight_one: None,
            sight_two: None,
            decay: Decimal::new(0, 6),
            burn: 0,
        };
    }
}
