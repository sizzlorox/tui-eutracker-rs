mod loadout;
mod logger;
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
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use session::{Session, Stopwatch};
use std::io;
use std::time::{Duration, Instant};
use std::{path::Path, sync::mpsc::channel};
use tui::backend::CrosstermBackend;
use tui::Terminal;

use crate::parser::{Base as ParserBase, Parser};
use crate::tracker::{Base as TrackerBase, Tracker};
use crate::ui::{TrackerUI, UI};

/*
   TODO:
   * Fix watch restarting on file change causing session file to be empty
   * Left off: Working on loadout table state
   * Next: Session list state
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
    let mut ui = TrackerUI::new(active_session_idx);

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
                    match key.code {
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
                                Loadout::new(format!("{}_loadout.json", date_string).as_str());
                                tracker.loadouts = Loadout::fetch();
                            }
                            _ => {}
                        },
                        KeyCode::Up => match ui.active_menu_item {
                            ui::MenuItem::Session => TrackerUI::previous_session(
                                &mut ui,
                                tracker.sessions.values().collect::<Vec<&Session>>(),
                            ),
                            ui::MenuItem::Loadout => TrackerUI::previous_loadout(
                                &mut ui,
                                tracker.loadouts.values().collect::<Vec<&Loadout>>(),
                            ),
                            _ => {}
                        },
                        KeyCode::Down => match ui.active_menu_item {
                            ui::MenuItem::Session => TrackerUI::next_session(
                                &mut ui,
                                tracker.sessions.values().collect::<Vec<&Session>>(),
                            ),
                            ui::MenuItem::Loadout => TrackerUI::next_loadout(
                                &mut ui,
                                tracker.loadouts.values().collect::<Vec<&Loadout>>(),
                            ),
                            _ => {}
                        },
                        KeyCode::Right => match ui.active_menu_item {
                            // TODO: need to rename current, then move selected to current
                            ui::MenuItem::Session => {
                                tracker.current_session.pause();
                                tracker.current_session.save();

                                tracker.sessions = Session::fetch();

                                let mut sessions_vec: Vec<&Session> =
                                    tracker.sessions.values().into_iter().collect();
                                sessions_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                                let mut new_session =
                                    sessions_vec[ui.session_list_state.selected().unwrap()].clone();
                                ui.active_session_idx = ui.session_list_state.selected();
                                new_session.start_time = Instant::now();

                                tracker
                                    .logs
                                    .push_front(format!("Selecting Session: {}", new_session.name));
                                tracker.current_session = new_session;
                            }
                            ui::MenuItem::Loadout => {
                                tracker.logs.push_front("Selecting Loadout".to_string());
                                tracker.current_session.loadout = tracker
                                    .loadouts
                                    .values()
                                    .into_iter()
                                    .collect::<Vec<&Loadout>>()
                                    [ui.loadout_table_state.selected().unwrap()]
                                .clone()
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
            last_tick = Instant::now();
        }
    }
}
