use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState, Tabs, Wrap,
    },
    Frame,
};

use crate::{
    session::Stopwatch,
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

pub struct TrackerUI {
    pub active_menu_item: MenuItem,
    menu_items: Vec<String>,
    loadout_table_state: TableState,
}

impl TrackerUI {
    pub fn new() -> TrackerUI {
        return TrackerUI {
            active_menu_item: MenuItem::Home,
            menu_items: vec!["Home", "Session", "Loadout", "Markup", "Options", "Quit"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            loadout_table_state: TableState::default(),
        };
    }
}

pub trait UI {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, tracker: &Tracker);
}

impl UI for TrackerUI {
    fn draw<B: Backend>(self: &mut Self, f: &mut Frame<B>, tracker: &Tracker) {
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
        let menu_section = self.get_menu_section(self.active_menu_item);
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

                let skills_section = TrackerUI::get_skills_section(tracker);
                let loot_section = TrackerUI::get_loot_section(tracker);
                let combat_section = TrackerUI::get_combat_section(tracker);

                f.render_widget(skills_section, body_chunks[0]);
                f.render_widget(loot_section, body_chunks[1]);
                f.render_widget(combat_section, body_chunks[2]);
            }
            MenuItem::Session => {
                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                    .split(chunks[1]);

                let session_list_section = TrackerUI::get_session_list_section(tracker);
                let session_details_section = TrackerUI::get_session_details_section(tracker);

                // TODO: Change to stateful list
                f.render_widget(session_list_section, body_chunks[0]);
                f.render_widget(session_details_section, body_chunks[1]);
            }
            MenuItem::Loadout => {
                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(chunks[1]);
                let loadouts_section = TrackerUI::get_loadouts_section(tracker);
                f.render_stateful_widget(
                    loadouts_section,
                    body_chunks[0],
                    &mut self.loadout_table_state,
                );
            }
            // MenuItem::Markup => {
            //     let markup_section = draw_markup_section(tracker);
            //     f.render_widget(markup_section, chunks[1]);
            // },
            // MenuItem::Options => {
            //     let options_section = draw_options_section(tracker);
            //     f.render_widget(options_section, chunks[1]);
            // },
            _ => {}
        }

        let logs_section = TrackerUI::get_logs_section(tracker);
        f.render_widget(logs_section, chunks[2]);
    }
}

pub trait Section {
    // COMMON
    fn get_menu_section<'a>(&'a self, active_menu_item: MenuItem) -> Tabs<'a>;
    fn get_logs_section<'a>(tracker: &'a Tracker) -> List<'a>;

    // HOME
    fn get_skills_section<'a>(tracker: &'a Tracker) -> Paragraph<'a>;
    fn get_loot_section<'a>(tracker: &'a Tracker) -> Paragraph<'a>;
    fn get_combat_section<'a>(tracker: &'a Tracker) -> Paragraph<'a>;

    // SESSION
    fn get_session_list_section<'a>(tracker: &'a Tracker) -> List<'a>;
    fn get_session_details_section<'a>(tracker: &'a Tracker) -> Paragraph<'a>;

    // LOADOUT
    fn get_loadouts_section<'a>(tracker: &'a Tracker) -> Table<'a>;
}

impl Section for TrackerUI {
    // COMMON
    fn get_menu_section<'a>(self: &'a Self, active_menu_item: MenuItem) -> Tabs<'a> {
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
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(Span::raw("|"));

        return tabs;
    }

    fn get_logs_section<'a>(tracker: &'a Tracker) -> List<'a> {
        let logs: Vec<ListItem> = tracker
            .logs
            .iter()
            .map(|log| ListItem::new(log.as_str()).style(Style::default().fg(Color::White)))
            .collect();
        let list: List = List::new(logs)
            .block(Block::default().title("Logs").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));

        return list;
    }

    // HOME
    fn get_skills_section<'a>(tracker: &'a Tracker) -> Paragraph<'a> {
        let total_exp_gain = Spans::from(Span::raw(format!(
            "Total Exp Gain: {}",
            tracker.current_session.stats.self_total_exp_gain
        )));
        let mut skill_items: Vec<Spans> = tracker
            .current_session
            .skill_map
            .iter()
            .map(|(_, skill)| Spans::from(Span::raw(format!("{}: {}", skill.name, skill.exp_gain))))
            .collect();
        let mut spans_vec = vec![total_exp_gain];
        spans_vec.append(&mut skill_items);

        let paragraph = Paragraph::new(spans_vec)
            .block(
                Block::default()
                    .title("Skills")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        return paragraph;
    }

    fn get_loot_section<'a>(tracker: &'a Tracker) -> Paragraph<'a> {
        let mu_profit = Spans::from(Span::raw(format!(
            "MU Profit: {} PED ({}%)",
            tracker.current_session.stats.mu_profit,
            Utils::get_percentage(
                tracker.current_session.stats.mu_profit,
                tracker.current_session.stats.total_cost
            )
        )));
        let tt_profit = Spans::from(Span::raw(format!(
            "TT Profit: {} PED ({}%)",
            tracker.current_session.stats.tt_profit,
            Utils::get_percentage(
                tracker.current_session.stats.tt_profit,
                tracker.current_session.stats.total_cost
            )
        )));
        let mut spans_vec = vec![mu_profit, tt_profit];
        let mut items_vec: Vec<Spans> = tracker
            .current_session
            .loot_map
            .iter()
            .map(|(_, loot)| {
                Spans::from(Span::raw(format!(
                    "{} (x{}): {} PED",
                    loot.name, loot.count, loot.mu_value
                )))
            })
            .collect();
        spans_vec.append(&mut items_vec);

        let paragraph = Paragraph::new(spans_vec)
            .block(
                Block::default()
                    .title("Loot")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        return paragraph;
    }

    fn get_combat_section<'a>(tracker: &'a Tracker) -> Paragraph<'a> {
        let self_total_damage = Spans::from(Span::raw(format!(
            "Total Damage: {}",
            tracker.current_session.stats.self_total_damage
        )));

        let paragraph: Paragraph = Paragraph::new(vec![self_total_damage])
            .block(
                Block::default()
                    .title("Combat")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        paragraph
    }

    // Session
    fn get_session_list_section<'a>(tracker: &'a Tracker) -> List<'a> {
        let session_items: Vec<ListItem> = tracker
            .sessions
            .iter()
            .map(|(name, _)| ListItem::new(name.as_str()).style(Style::default().fg(Color::White)))
            .collect();

        let list: List = List::new(session_items)
            .block(Block::default().title("Sessions").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));

        list
    }

    fn get_session_details_section<'a>(tracker: &'a Tracker) -> Paragraph<'a> {
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
                    .style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        paragraph
    }

    // Loadout
    fn get_loadouts_section<'a>(tracker: &'a Tracker) -> Table<'a> {
        let headers = vec![
            "Name",
            "Weapon",
            "Amp",
            "Scope",
            "Sight 1",
            "Sight 2",
            "Decay",
            "Ammo Burn",
        ];

        let header = Row::new(headers)
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(1);
        let rows: Vec<Row> = tracker
            .loadouts
            .iter()
            .map(|(_, loadout)| {
                Row::new(vec![
                    Cell::from(loadout.name.as_str()),
                    Cell::from(loadout.weapon.as_deref().unwrap_or("None")),
                    Cell::from(loadout.amp.as_deref().unwrap_or("None")),
                    Cell::from(loadout.scope.as_deref().unwrap_or("None")),
                    Cell::from(loadout.sight_one.as_deref().unwrap_or("None")),
                    Cell::from(loadout.sight_two.as_deref().unwrap_or("None")),
                    Cell::from(loadout.decay.to_string()),
                    Cell::from(loadout.burn.to_string()),
                ])
                .style(Style::default().fg(Color::White))
            })
            .collect();

        let table = Table::new(rows)
            .header(header)
            .block(
                Block::default()
                    .title("Loadouts")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .highlight_symbol(">> ")
            .widths(&[
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
            ]);

        table
    }
}
