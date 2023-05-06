use std::{collections::HashMap, ops::Mul, usize};

use rust_decimal::Decimal;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState, Tabs,
        Wrap,
    },
    Frame,
};

use crate::{
    loadout::Loadout,
    markup::Markup,
    session::{Session, SessionLoot, SessionSkill, Stopwatch},
    tracker::Tracker,
    utils::{Helpers, Utils},
};

#[derive(Clone, Copy)]
pub enum MenuItem {
    Home,
    Session,
    Loadout,
    Markup,
    Options,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Session => 1,
            MenuItem::Loadout => 2,
            MenuItem::Markup => 3,
            MenuItem::Options => 4,
        }
    }
}

#[derive(Clone, Copy)]
pub enum HomeSection {
    None,
    Skills,
    LootSummary,
    LootDetails,
    CombatSelf,
    CombatTarget,
}

impl HomeSection {
    fn count() -> usize {
        return 6;
    }
}

impl From<HomeSection> for usize {
    fn from(input: HomeSection) -> usize {
        match input {
            HomeSection::None => 0,
            HomeSection::Skills => 1,
            HomeSection::LootSummary => 2,
            HomeSection::LootDetails => 3,
            HomeSection::CombatSelf => 4,
            HomeSection::CombatTarget => 5,
        }
    }
}

#[derive(PartialEq)]
pub enum EditableTableMode {
    View,
    Edit,
}

impl From<EditableTableMode> for usize {
    fn from(input: EditableTableMode) -> usize {
        match input {
            EditableTableMode::View => 0,
            EditableTableMode::Edit => 1,
        }
    }
}

pub trait EditableTable {
    fn toggle_mode(&mut self, markups: &mut HashMap<String, Markup>, active_idx: usize);
}

pub struct EditableTableState {
    pub mode: EditableTableMode,
    pub input: String,
}

impl EditableTable for EditableTableState {
    fn toggle_mode(&mut self, markups: &mut HashMap<String, Markup>, active_idx: usize) {
        self.mode = match self.mode {
            EditableTableMode::View => EditableTableMode::Edit,
            EditableTableMode::Edit => {
                let mut markups_vec: Vec<Markup> = markups.values().cloned().collect();
                markups_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                markups_vec[active_idx].value = Decimal::from_str_exact(self.input.as_str())
                    .unwrap_or(Decimal::ONE_HUNDRED)
                    .checked_div(Decimal::ONE_HUNDRED)
                    .unwrap_or(Decimal::ONE_HUNDRED);
                markups.insert(
                    markups_vec[active_idx].clone().name,
                    markups_vec[active_idx].clone(),
                );
                self.input = String::new();
                EditableTableMode::View
            }
        }
    }
}

pub trait SectionState {
    fn next(&mut self);
    fn previous(&mut self);
    fn select(&mut self, value: usize);
    fn scroll_up(&mut self);
    fn scroll_down(&mut self);
}

pub struct HomeSectionState {
    pub active_section: HomeSection,
    pub skills_scroll_offset: u16,
    pub loot_summary_scroll_offset: u16,
    pub loot_details_scroll_offset: u16,
    pub combat_self_scroll_offset: u16,
    pub combat_target_scroll_offset: u16,
}

impl SectionState for HomeSectionState {
    fn next(&mut self) {
        let i = match self.active_section.into() {
            Some(i) => {
                let idx: usize = i.into();
                if idx >= HomeSection::count() - 1 {
                    0
                } else {
                    idx + 1
                }
            }
            None => 0,
        };
        self.select(i);
    }
    fn previous(&mut self) {
        let i = match self.active_section.into() {
            Some(i) => {
                let idx: usize = i.into();
                if idx == 0 {
                    HomeSection::count() - 1
                } else {
                    idx - 1
                }
            }
            None => 0,
        };
        self.select(i);
    }
    fn select(&mut self, value: usize) {
        self.active_section = match value {
            0 => HomeSection::None,
            1 => HomeSection::Skills,
            2 => HomeSection::LootSummary,
            3 => HomeSection::LootDetails,
            4 => HomeSection::CombatSelf,
            5 => HomeSection::CombatTarget,
            _ => HomeSection::None,
        };
    }
    fn scroll_up(&mut self) {
        match self.active_section {
            HomeSection::Skills => {
                if self.skills_scroll_offset > 0 {
                    self.skills_scroll_offset -= 1;
                }
            }
            HomeSection::LootSummary => {
                if self.loot_summary_scroll_offset > 0 {
                    self.loot_summary_scroll_offset -= 1;
                }
            }
            HomeSection::LootDetails => {
                if self.loot_details_scroll_offset > 0 {
                    self.loot_details_scroll_offset -= 1;
                }
            }
            HomeSection::CombatSelf => {
                if self.combat_self_scroll_offset > 0 {
                    self.combat_self_scroll_offset -= 1;
                }
            }
            HomeSection::CombatTarget => {
                if self.combat_target_scroll_offset > 0 {
                    self.combat_target_scroll_offset -= 1;
                }
            }
            _ => {}
        }
    }
    fn scroll_down(&mut self) {
        match self.active_section {
            HomeSection::Skills => {
                self.skills_scroll_offset += 1;
            }
            HomeSection::LootSummary => {
                self.loot_summary_scroll_offset += 1;
            }
            HomeSection::LootDetails => {
                self.loot_details_scroll_offset += 1;
            }
            HomeSection::CombatSelf => {
                self.combat_self_scroll_offset += 1;
            }
            HomeSection::CombatTarget => {
                self.combat_target_scroll_offset += 1;
            }
            _ => {}
        }
    }
}

pub struct TrackerUI {
    menu_items: Vec<String>,

    pub active_menu_item: MenuItem,
    pub active_session_idx: Option<usize>,
    pub active_loadout_idx: Option<usize>,
    pub home_section_state: HomeSectionState,
    pub session_list_state: ListState,
    pub loadout_table_state: TableState,
    pub markup_table_state: TableState,
    pub markup_editable_table_state: EditableTableState,
}

impl TrackerUI {
    pub fn new(active_session_idx: Option<usize>, active_loadout_idx: Option<usize>) -> TrackerUI {
        let mut session_list_state = ListState::default();
        session_list_state.select(active_session_idx);
        let mut loadout_table_state = TableState::default();
        loadout_table_state.select(active_loadout_idx);
        return TrackerUI {
            home_section_state: HomeSectionState {
                active_section: HomeSection::None,
                skills_scroll_offset: 0,
                loot_summary_scroll_offset: 0,
                loot_details_scroll_offset: 0,
                combat_self_scroll_offset: 0,
                combat_target_scroll_offset: 0,
            },
            active_menu_item: MenuItem::Home,
            menu_items: vec!["Home", "Session", "Loadout", "Markup", "Options", "Quit"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            active_session_idx,
            active_loadout_idx,
            session_list_state,
            loadout_table_state,
            markup_table_state: TableState::default(),
            markup_editable_table_state: EditableTableState {
                mode: EditableTableMode::View,
                input: String::new(),
            },
        };
    }
    pub fn next_session(&mut self, items: Vec<&Session>) {
        if items.len() == 0 {
            return;
        }
        let i = match self.session_list_state.selected() {
            Some(i) => {
                if i >= items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.session_list_state.select(Some(i));
    }
    pub fn previous_session(&mut self, items: Vec<&Session>) {
        if items.len() == 0 {
            return;
        }
        let i = match self.session_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.session_list_state.select(Some(i));
    }
    pub fn next_loadout(&mut self, items: Vec<&Loadout>) {
        if items.len() == 0 {
            return;
        }
        let i = match self.loadout_table_state.selected() {
            Some(i) => {
                if i >= items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.loadout_table_state.select(Some(i));
    }
    pub fn previous_loadout(&mut self, items: Vec<&Loadout>) {
        if items.len() == 0 {
            return;
        }
        let i = match self.loadout_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.loadout_table_state.select(Some(i));
    }
    pub fn next_markup(&mut self, items: Vec<&Markup>) {
        if items.len() == 0 {
            return;
        }
        let i = match self.markup_table_state.selected() {
            Some(i) => {
                if i >= items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.markup_table_state.select(Some(i));
    }
    pub fn previous_markup(&mut self, items: Vec<&Markup>) {
        if items.len() == 0 {
            return;
        }
        let i = match self.markup_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.markup_table_state.select(Some(i));
    }
}

pub trait UI {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, tracker: &Tracker);
}

impl UI for TrackerUI {
    fn draw<B: Backend>(self: &mut Self, f: &mut Frame<B>, tracker: &Tracker) {
        let mut ui_color = Color::Cyan;
        if tracker.current_session.is_active {
            ui_color = Color::Green;
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ]
                .as_ref(),
            )
            .split(f.size());
        let menu_section = self.get_menu_section(ui_color, self.active_menu_item);
        f.render_widget(menu_section, chunks[0]);

        match self.active_menu_item {
            MenuItem::Home => {
                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(33),
                            Constraint::Percentage(33),
                            Constraint::Percentage(33),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[1]);

                let skills_section = TrackerUI::get_skills_section(
                    match self.home_section_state.active_section {
                        HomeSection::Skills => Color::Yellow,
                        _ => ui_color,
                    },
                    tracker,
                    self.home_section_state.skills_scroll_offset,
                );
                let loot_summary_section = TrackerUI::get_summary_loot_section(
                    match self.home_section_state.active_section {
                        HomeSection::LootSummary => Color::Yellow,
                        _ => ui_color,
                    },
                    tracker,
                    self.home_section_state.loot_summary_scroll_offset,
                );
                let loot_details_section = TrackerUI::get_details_loot_section(
                    match self.home_section_state.active_section {
                        HomeSection::LootDetails => Color::Yellow,
                        _ => ui_color,
                    },
                    tracker,
                    self.home_section_state.loot_details_scroll_offset,
                );

                let loot_body_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                    .split(body_chunks[1]);

                let combat_body_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(body_chunks[2]);

                let self_combat_section = TrackerUI::get_self_combat_section(
                    match self.home_section_state.active_section {
                        HomeSection::CombatSelf => Color::Yellow,
                        _ => ui_color,
                    },
                    tracker,
                    self.home_section_state.combat_self_scroll_offset,
                );
                let target_combat_section = TrackerUI::get_target_combat_section(
                    match self.home_section_state.active_section {
                        HomeSection::CombatTarget => Color::Yellow,
                        _ => ui_color,
                    },
                    tracker,
                    self.home_section_state.combat_target_scroll_offset,
                );

                f.render_widget(skills_section, body_chunks[0]);
                f.render_widget(loot_summary_section, loot_body_chunks[0]);
                f.render_widget(loot_details_section, loot_body_chunks[1]);
                f.render_widget(self_combat_section, combat_body_chunks[0]);
                f.render_widget(target_combat_section, combat_body_chunks[1]);
            }
            MenuItem::Session => {
                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                    .split(chunks[1]);

                let session_list_section = TrackerUI::get_session_list_section(
                    ui_color,
                    tracker,
                    self.active_session_idx.unwrap(),
                );
                let session_details_section =
                    TrackerUI::get_session_details_section(ui_color, tracker);

                f.render_stateful_widget(
                    session_list_section,
                    body_chunks[0],
                    &mut self.session_list_state,
                );
                f.render_widget(session_details_section, body_chunks[1]);
            }
            MenuItem::Loadout => {
                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(chunks[1]);
                let loadouts_section = TrackerUI::get_loadouts_section(
                    ui_color,
                    tracker,
                    self.active_loadout_idx.unwrap(),
                );
                f.render_stateful_widget(
                    loadouts_section,
                    body_chunks[0],
                    &mut self.loadout_table_state,
                );
            }
            MenuItem::Markup => {
                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(chunks[1]);
                let markup_section = TrackerUI::get_markups_section(
                    ui_color,
                    tracker,
                    &mut self.markup_editable_table_state,
                    self.markup_table_state.selected().unwrap_or(0),
                );
                f.render_stateful_widget(
                    markup_section,
                    body_chunks[0],
                    &mut self.markup_table_state,
                );
            }
            // MenuItem::Options => {
            //     let options_section = draw_options_section(tracker);
            //     f.render_widget(options_section, chunks[1]);
            // },
            _ => {}
        }

        let logs_section = TrackerUI::get_logs_section(ui_color, tracker);
        f.render_widget(logs_section, chunks[2]);
    }
}

pub trait Section {
    // COMMON
    fn get_menu_section<'a>(&'a self, ui_color: Color, active_menu_item: MenuItem) -> Tabs<'a>;
    fn get_logs_section<'a>(ui_color: Color, tracker: &'a Tracker) -> List<'a>;

    // HOME
    fn get_skills_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a>;
    fn get_summary_loot_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a>;
    fn get_details_loot_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a>;
    fn get_self_combat_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a>;
    fn get_target_combat_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a>;

    // SESSION
    fn get_session_list_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        active_session_idx: usize,
    ) -> List<'a>;
    fn get_session_details_section<'a>(ui_color: Color, tracker: &'a Tracker) -> Paragraph<'a>;

    // LOADOUT
    fn get_loadouts_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        active_loadout_idx: usize,
    ) -> Table<'a>;

    // MARKUP
    fn get_markups_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        state: &'a mut EditableTableState,
        active_idx: usize,
    ) -> Table<'a>;
}

impl Section for TrackerUI {
    // COMMON
    fn get_menu_section<'a>(
        self: &'a Self,
        ui_color: Color,
        active_menu_item: MenuItem,
    ) -> Tabs<'a> {
        let menu = self
            .menu_items
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Spans::from(vec![
                    Span::styled(
                        first,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled(rest, Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let tabs = Tabs::new(menu)
            .select(active_menu_item.into())
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(ui_color))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(Span::raw("|"));

        return tabs;
    }

    fn get_logs_section<'a>(ui_color: Color, tracker: &'a Tracker) -> List<'a> {
        let logs: Vec<ListItem> = tracker
            .logs
            .iter()
            .map(|log| ListItem::new(log.as_str()).style(Style::default().fg(Color::White)))
            .collect();
        let list: List = List::new(logs)
            .block(Block::default().title("Logs").borders(Borders::ALL))
            .style(Style::default().fg(ui_color));

        return list;
    }

    // HOME
    fn get_skills_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a> {
        let total_exp_gain = Spans::from(Span::raw(format!(
            "Total Exp Gain: {}",
            tracker.current_session.stats.self_total_exp_gain
        )));
        let mut sorted_skills_vec: Vec<&SessionSkill> = tracker
            .current_session
            .skill_map
            .values()
            .into_iter()
            .collect();
        sorted_skills_vec.sort_by(|a, b| b.exp_gain.cmp(&a.exp_gain));
        let mut skill_items: Vec<Spans> = sorted_skills_vec
            .iter()
            .map(|skill| Spans::from(Span::raw(format!("{}: {}", skill.name, skill.exp_gain))))
            .collect();
        let mut spans_vec = vec![total_exp_gain];
        spans_vec.append(&mut skill_items);

        let paragraph = Paragraph::new(spans_vec)
            .block(
                Block::default()
                    .title("Skills")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(ui_color)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset, 0))
            .style(Style::default().fg(Color::White));

        return paragraph;
    }

    fn get_summary_loot_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a> {
        let total_cost = Spans::from(Span::raw(format!(
            "Total Cost: {} PED",
            tracker.current_session.stats.total_cost.trunc_with_scale(4),
        )));
        let mu_profit_value: Decimal = tracker
            .current_session
            .loot_map
            .values()
            .collect::<Vec<&SessionLoot>>()
            .iter()
            .fold(Decimal::new(0, 0), |a, l| {
                a + (l.tt_value * tracker.markups.get(l.name.as_str()).unwrap().value)
            });
        let mu_profit = Spans::from(Span::raw(format!(
            "MU Profit: {} PED ({}%)",
            (mu_profit_value - tracker.current_session.stats.total_cost).trunc_with_scale(4),
            Utils::get_percentage(mu_profit_value, tracker.current_session.stats.total_cost)
        )));
        let ped_per_hour = Spans::from(Span::raw(format!(
            "PED/Hour: {} PED",
            mu_profit_value
                .checked_div(Decimal::from(tracker.current_session.elapsed().as_secs()))
                .unwrap_or(Decimal::ZERO)
                .mul(Decimal::from(3600))
                .trunc_with_scale(4)
        )));
        let cost_per_hour = Spans::from(Span::raw(format!(
            "Cost/Hour: {} PED",
            tracker
                .current_session
                .stats
                .total_cost
                .checked_div(Decimal::from(tracker.current_session.elapsed().as_secs()))
                .unwrap_or(Decimal::ZERO)
                .mul(Decimal::from(3600))
                .trunc_with_scale(4)
        )));
        let tt_profit = Spans::from(Span::raw(format!(
            "TT Profit: {} PED ({}%)",
            (tracker.current_session.stats.tt_profit - tracker.current_session.stats.total_cost)
                .trunc_with_scale(4),
            Utils::get_percentage(
                tracker.current_session.stats.tt_profit,
                tracker.current_session.stats.total_cost
            )
        )));
        let spans_vec = vec![
            ped_per_hour,
            cost_per_hour,
            total_cost,
            mu_profit,
            tt_profit,
        ];

        let paragraph = Paragraph::new(spans_vec)
            .block(
                Block::default()
                    .title("Loot Summary")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(ui_color)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset, 0))
            .style(Style::default().fg(Color::White));

        return paragraph;
    }

    fn get_details_loot_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a> {
        let mut sorted_item_vec: Vec<&SessionLoot> = tracker
            .current_session
            .loot_map
            .values()
            .into_iter()
            .collect();
        sorted_item_vec.sort_by(|a, b| b.tt_value.cmp(&a.tt_value));
        let items_vec: Vec<Spans> = sorted_item_vec
            .iter()
            .map(|loot| {
                Spans::from(Span::raw(format!(
                    "{} (x{}): {} PED",
                    loot.name,
                    loot.count,
                    (loot.tt_value * tracker.markups.get(loot.name.as_str()).unwrap().value)
                        .trunc_with_scale(4)
                )))
            })
            .collect();

        let paragraph = Paragraph::new(items_vec)
            .block(
                Block::default()
                    .title("Loot Drops")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(ui_color)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset, 0))
            .style(Style::default().fg(Color::White));

        return paragraph;
    }

    fn get_self_combat_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a> {
        let self_total_damage = Spans::from(Span::raw(format!(
            "Total Damage: {}",
            tracker.current_session.stats.self_total_damage
        )));
        let self_total_heal = Spans::from(Span::raw(format!(
            "Total Heal: {}",
            tracker.current_session.stats.self_total_heal
        )));
        let self_total_shots = Spans::from(Span::raw(format!(
            "Total Shots: {}",
            tracker.current_session.stats.self_attack_count
        )));
        let self_crit_chance = Spans::from(Span::raw(format!(
            "Crit Chance: {}%",
            Utils::get_percentage(
                Decimal::from(tracker.current_session.stats.self_crit_count),
                Decimal::from(tracker.current_session.stats.self_attack_count),
            )
        )));
        let self_miss = Spans::from(Span::raw(format!(
            "Miss: {}%",
            Utils::get_percentage(
                Decimal::from(tracker.current_session.stats.self_attack_miss_count),
                Decimal::from(tracker.current_session.stats.self_attack_count),
            )
        )));
        let self_deflected = Spans::from(Span::raw(format!(
            "Deflected: {}%",
            Utils::get_percentage(
                Decimal::from(tracker.current_session.stats.self_deflect_count),
                Decimal::from(tracker.current_session.stats.target_attack_count),
            )
        )));
        let self_death_count = Spans::from(Span::raw(format!(
            "Death Count: {}",
            tracker.current_session.stats.self_death_count,
        )));

        let paragraph: Paragraph = Paragraph::new(vec![
            self_total_shots,
            self_total_damage,
            self_total_heal,
            self_crit_chance,
            self_miss,
            self_deflected,
            self_death_count,
        ])
        .block(
            Block::default()
                .title("Player")
                .borders(Borders::ALL)
                .style(Style::default().fg(ui_color)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .scroll((scroll_offset, 0))
        .style(Style::default().fg(Color::White));

        paragraph
    }

    fn get_target_combat_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        scroll_offset: u16,
    ) -> Paragraph<'a> {
        let target_total_damage = Spans::from(Span::raw(format!(
            "Total Damage: {}",
            tracker.current_session.stats.target_total_damage
        )));
        let target_attack_count = Spans::from(Span::raw(format!(
            "Total Attacks: {}",
            tracker.current_session.stats.target_attack_count
        )));
        let target_miss = Spans::from(Span::raw(format!(
            "Miss: {}%",
            Utils::get_percentage(
                Decimal::from(tracker.current_session.stats.self_evade_count),
                Decimal::from(tracker.current_session.stats.target_attack_count),
            )
        )));

        let paragraph: Paragraph =
            Paragraph::new(vec![target_total_damage, target_attack_count, target_miss])
                .block(
                    Block::default()
                        .title("Target")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(ui_color)),
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .scroll((scroll_offset, 0))
                .style(Style::default().fg(Color::White));

        paragraph
    }

    // Session
    fn get_session_list_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        active_session_idx: usize,
    ) -> List<'a> {
        let mut sessions_vec: Vec<&Session> = tracker.sessions.values().into_iter().collect();
        sessions_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let session_items: Vec<ListItem> = sessions_vec
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                if idx == active_session_idx {
                    return ListItem::new(s.name.as_str()).style(Style::default().fg(Color::Green));
                }
                ListItem::new(s.name.as_str()).style(Style::default().fg(Color::White))
            })
            .collect();

        let list: List = List::new(session_items)
            .block(Block::default().title("Sessions").borders(Borders::ALL))
            .style(Style::default().fg(ui_color))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        list
    }

    fn get_session_details_section<'a>(ui_color: Color, tracker: &'a Tracker) -> Paragraph<'a> {
        let elapsed_time = Spans::from(Span::raw(format!(
            "Elapsed Time: {}",
            tracker.current_session.pretty_elapsed()
        )));
        let is_running = Spans::from(Span::raw(format!(
            "Running: {}",
            tracker.current_session.is_active
        )));
        let paragraph_vec = vec![elapsed_time, is_running];

        let paragraph = Paragraph::new(paragraph_vec)
            .block(
                Block::default()
                    .title("Session Details")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(ui_color)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        paragraph
    }

    // Loadout
    fn get_loadouts_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        active_loadout_idx: usize,
    ) -> Table<'a> {
        let headers = vec![
            "Name",
            "Weapon",
            "Amp",
            "Scope",
            "Sight 1",
            "Sight 2",
            "Decay",
            "Ammo Burn",
            "Cost per Shot",
        ];

        let header = Row::new(headers)
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(1);
        let mut loadouts_vec: Vec<&Loadout> = tracker.loadouts.values().into_iter().collect();
        loadouts_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let rows: Vec<Row> = loadouts_vec
            .iter()
            .enumerate()
            .map(|(idx, loadout)| {
                let rows_vec = vec![
                    Cell::from(loadout.name.as_str()),
                    Cell::from(loadout.weapon.as_deref().unwrap_or("None")),
                    Cell::from(loadout.amp.as_deref().unwrap_or("None")),
                    Cell::from(loadout.scope.as_deref().unwrap_or("None")),
                    Cell::from(loadout.sight_one.as_deref().unwrap_or("None")),
                    Cell::from(loadout.sight_two.as_deref().unwrap_or("None")),
                    Cell::from((loadout.decay.mul(Decimal::new(1, 2))).to_string()),
                    Cell::from(loadout.burn.to_string()),
                    Cell::from(
                        Decimal::from(loadout.burn)
                            .checked_div(
                                Decimal::from(10000) + loadout.decay.mul(Decimal::new(1, 2)),
                            )
                            .unwrap_or(Decimal::ZERO)
                            .trunc_with_scale(6)
                            .to_string(),
                    ),
                ];
                if idx == active_loadout_idx {
                    return Row::new(rows_vec).style(Style::default().fg(Color::Green));
                }
                Row::new(rows_vec).style(Style::default().fg(Color::White))
            })
            .collect();

        let table = Table::new(rows)
            .header(header)
            .block(
                Block::default()
                    .title("Loadouts")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(ui_color)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
            .widths(&[
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
            ]);

        table
    }

    fn get_markups_section<'a>(
        ui_color: Color,
        tracker: &'a Tracker,
        state: &'a mut EditableTableState,
        active_idx: usize,
    ) -> Table<'a> {
        let headers = vec!["Name", "Markup"];
        let header = Row::new(headers)
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(1);
        let mut markups_vec: Vec<&Markup> = tracker.markups.values().into_iter().collect();
        markups_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let rows: Vec<Row> = markups_vec
            .iter()
            .enumerate()
            .map(|(idx, loadout)| {
                let mut value_cell =
                    Cell::from(format!("{}%", loadout.value.mul(Decimal::ONE_HUNDRED)));
                if active_idx == idx && state.mode == EditableTableMode::Edit {
                    value_cell = Cell::from(state.input.as_ref());
                }
                let rows_vec = vec![Cell::from(loadout.name.as_str()), value_cell];
                Row::new(rows_vec).style(Style::default().fg(Color::White))
            })
            .collect();

        let table = Table::new(rows)
            .header(header)
            .block(
                Block::default()
                    .title("Loadouts")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(ui_color)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);

        table
    }
}
