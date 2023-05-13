mod loadout;
mod logger;
mod markup;
mod parser;
mod session;
mod tracker;
mod ui;
mod utils;

use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
use loadout::Loadout;
use markup::Markup;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use session::{Session, Stopwatch};
use std::io;
use std::time::{Duration, Instant};
use std::{path::Path, sync::mpsc::channel};
use tui::backend::CrosstermBackend;
use tui::Terminal;
use ui::{EditableTable, EditableTableMode};

use crate::parser::{Base as ParserBase, Parser};
use crate::tracker::{Base as TrackerBase, Tracker};
use crate::ui::{SectionState, TrackerUI, UI};

/*
   TODO:
   * Fix watch restarting on file change causing session file to be empty
   * Add PVP section, KDR and stuff ,{} killed {} using a {}., {} DISABLED {} using a {}.,
   * Clean up UI module
   * Fix adding new session does not update with correct active index
   * Loot items value seems a bit off? Maybe it's getting rounded? wtf
   * Solve overflow from serde_millis, maybe write own
*/

fn main() {
    let log_path = "C:\\Users\\sizzl\\OneDrive\\Documents\\Entropia Universe\\chat.log";
    let mut parser = Parser::new();
    let (tx, parser_receiver) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    watcher
        .watch(Path::new(log_path), RecursiveMode::NonRecursive)
        .unwrap();

    let mut tracker = Tracker::new(String::from("Aardvark sizz-lorr Nolin"));
    let mut sessions_vec: Vec<&Session> = tracker.sessions.values().into_iter().collect();
    sessions_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let active_session_idx = sessions_vec
        .iter()
        .position(|&s| s.name == tracker.current_session.name);
    let mut loadouts_vec: Vec<&Loadout> = tracker.loadouts.values().into_iter().collect();
    loadouts_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let active_loadout_idx = loadouts_vec
        .iter()
        .position(|&s| s.name == tracker.current_session.loadout.name);

    let mut ui = TrackerUI::new(active_session_idx, active_loadout_idx);

    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();

    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui.draw(f, &tracker)).unwrap();

        while let Ok(_) = parser_receiver.try_recv() {
            if let Some(lines) = parser.get_lines_to_parse(log_path) {
                for line in lines {
                    if let Some(log) = parser.parse(&line) {
                        tracker.track(log);
                    }
                }
            }
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    // EDITING
                    match ui.active_menu_item {
                        ui::MenuItem::Markup => match ui.markup_editable_table_state.mode {
                            EditableTableMode::Edit => match key.code {
                                KeyCode::Char(c) => {
                                    ui.markup_editable_table_state.input.push(c);
                                }
                                KeyCode::Backspace => {
                                    ui.markup_editable_table_state.input.pop();
                                }
                                _ => {}
                            },
                            _ => {}
                        },
                        _ => {}
                    }

                    match key.code {
                        KeyCode::Enter => match ui.active_menu_item {
                            ui::MenuItem::Markup => ui.markup_editable_table_state.toggle_mode(
                                &mut tracker.markups,
                                ui.markup_table_state.selected().unwrap_or(0),
                            ),
                            _ => {}
                        },
                        KeyCode::Char('h') => ui.active_menu_item = ui::MenuItem::Home,
                        KeyCode::Char('s') => ui.active_menu_item = ui::MenuItem::Session,
                        KeyCode::Char('l') => ui.active_menu_item = ui::MenuItem::Loadout,
                        KeyCode::Char('m') => ui.active_menu_item = ui::MenuItem::Markup,
                        KeyCode::Char('o') => ui.active_menu_item = ui::MenuItem::Options,
                        KeyCode::Char('n') => match ui.active_menu_item {
                            ui::MenuItem::Session => {
                                tracker.logs.push_front("Creating New Session".to_string());
                                let date_string = Local::now().format("%Y-%m-%d_%H-%M-%S");
                                Session::new(format!("{}_session.json", date_string).as_str());
                                tracker.sessions = Session::fetch();
                            }
                            ui::MenuItem::Loadout => {
                                tracker.logs.push_front("Creating New Loadout".to_string());
                                let date_string = Local::now().format("%Y-%m-%d_%H-%M-%S");
                                Loadout::new(date_string.to_string().as_str());
                                tracker.loadouts = Loadout::fetch();
                            }
                            _ => {}
                        },
                        KeyCode::Up => match ui.active_menu_item {
                            ui::MenuItem::Home => ui.home_section_state.scroll_up(),
                            ui::MenuItem::Session => TrackerUI::previous_session(
                                &mut ui,
                                tracker.sessions.values().collect::<Vec<&Session>>(),
                            ),
                            ui::MenuItem::Loadout => TrackerUI::previous_loadout(
                                &mut ui,
                                tracker.loadouts.values().collect::<Vec<&Loadout>>(),
                            ),
                            ui::MenuItem::Markup => TrackerUI::previous_markup(
                                &mut ui,
                                tracker.markups.values().collect::<Vec<&Markup>>(),
                            ),
                            _ => {}
                        },
                        KeyCode::Down => match ui.active_menu_item {
                            ui::MenuItem::Home => ui.home_section_state.scroll_down(),
                            ui::MenuItem::Session => TrackerUI::next_session(
                                &mut ui,
                                tracker.sessions.values().collect::<Vec<&Session>>(),
                            ),
                            ui::MenuItem::Loadout => TrackerUI::next_loadout(
                                &mut ui,
                                tracker.loadouts.values().collect::<Vec<&Loadout>>(),
                            ),
                            ui::MenuItem::Markup => TrackerUI::next_markup(
                                &mut ui,
                                tracker.markups.values().collect::<Vec<&Markup>>(),
                            ),
                            _ => {}
                        },
                        KeyCode::Left => match ui.active_menu_item {
                            ui::MenuItem::Home => ui.home_section_state.previous(),
                            _ => {}
                        },
                        KeyCode::Right => match ui.active_menu_item {
                            ui::MenuItem::Home => ui.home_section_state.next(),
                            ui::MenuItem::Session => {
                                let selected_idx = ui.session_list_state.selected().unwrap();
                                if selected_idx == ui.active_session_idx.unwrap() {
                                    tracker
                                        .logs
                                        .push_front("Session already selected".to_string());
                                    continue;
                                }

                                if tracker.current_session.is_active {
                                    tracker.current_session.pause();
                                }
                                tracker.current_session.save();

                                tracker.sessions = Session::fetch();

                                let mut sessions_vec: Vec<&Session> =
                                    tracker.sessions.values().into_iter().collect();
                                sessions_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                                let new_session = sessions_vec[selected_idx].clone();
                                ui.active_session_idx = ui.session_list_state.selected();

                                // Set active loadout idx
                                let mut loadouts_vec: Vec<&Loadout> =
                                    tracker.loadouts.values().into_iter().collect();
                                loadouts_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                                let active_loadout_idx = loadouts_vec
                                    .iter()
                                    .position(|&s| s.name == new_session.loadout.name);
                                ui.active_loadout_idx = active_loadout_idx;

                                tracker
                                    .logs
                                    .push_front(format!("Selecting Session: {}", new_session.name));
                                tracker.current_session = new_session;
                            }
                            ui::MenuItem::Loadout => {
                                let selected_idx = ui.loadout_table_state.selected().unwrap();
                                if selected_idx == ui.active_loadout_idx.unwrap() {
                                    tracker
                                        .logs
                                        .push_front("Loadout already selected".to_string());
                                    continue;
                                }
                                tracker.current_session.loadout.save();

                                tracker.loadouts = Loadout::fetch();

                                let mut loadouts_vec: Vec<&Loadout> =
                                    tracker.loadouts.values().into_iter().collect();
                                loadouts_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                                let new_loadout = loadouts_vec[selected_idx].clone();
                                ui.active_loadout_idx = ui.loadout_table_state.selected();

                                tracker
                                    .logs
                                    .push_front(format!("Selecting Loadout: {}", new_loadout.name));
                                tracker.current_session.loadout = new_loadout;
                            }
                            _ => {}
                        },
                        KeyCode::Char('p') => match tracker.current_session.is_active {
                            true => {
                                tracker.logs.push_front("Stopping Session".to_string());
                                tracker.current_session.pause();
                            }
                            false => {
                                tracker.logs.push_front("Starting Session".to_string());
                                tracker.current_session.start();
                            }
                        },
                        KeyCode::Char('q') => {
                            disable_raw_mode().unwrap();
                            terminal.show_cursor().unwrap();
                            terminal.clear().unwrap();
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            // Tracker ontick?
            tracker.current_session.save();
            tracker.current_session.loadout.save();
            Markup::save(tracker.markups.clone());
            last_tick = Instant::now();
        }
    }
}
