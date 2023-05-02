mod parser;
mod logger;
mod session;
mod loadout;
mod tracker;

use std::{sync::mpsc::channel, path::Path};
use notify::{RecommendedWatcher, Watcher, Config, RecursiveMode};

use crate::parser::{Parser, Base as ParserBase};
use crate::tracker::{Tracker, Base as TrackerBase};


fn main() {
    let log_path = "C:\\Users\\sizzl\\OneDrive\\Documents\\Entropia Universe\\chat.log";
    let mut parser = Parser::new();
    let (tx, parser_receiver) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    watcher.watch(Path::new(log_path), RecursiveMode::NonRecursive).unwrap();

    let mut tracker = Tracker::new();
    
    loop {
        match parser_receiver.try_recv() {
            Ok(_) => {
                if let Some(lines) = parser.get_lines_to_parse(log_path) {  
                    for line in lines {
                        if let Some(log) = parser.parse(&line) {
                            println!("{} ---- {}", log.line, log.values.join(", "));
                            tracker.track(log);
                        }
                    }
                }
            },
            Err(_) => {
                continue;
            }
        }
    }
}
