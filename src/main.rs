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
    let mut ui = TrackerUI::new();

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
                        KeyCode::Char('n') => {
                            let date_string = Local::now().format("%Y-%m-%d_%H-%M-%S");
                            tracker
                                .current_session
                                .export(format!("{}_session.json", date_string).as_str());
                            tracker.current_session = Session::new("current_session.json");
                            tracker.sessions = Session::fetch();
                        }
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
