use std::{fs::File, io::{BufReader, BufRead}};
use regex::{RegexSet, Regex};

use crate::logger::{Log, Logger, LogEvent};


pub trait Base<'a> {
    fn get_lines_to_parse(&mut self, log_path: &str) -> Option<Vec<String>>;
    fn parse(&'a self, line: &'a str) -> Option<Log>;
}

pub struct Parser<'a> {
    logger: Logger<'a>,
    regex_set: RegexSet,
    last_line: usize,
}

impl<'a> Parser<'a> {
    pub fn new() -> Parser<'a> {
        let logger = Logger::new();
        let mut regex_vec: Vec<_> = logger.log_events.iter().collect();
        regex_vec.sort_by(|a, b| a.0.cmp(&b.0));
        let regex_slice = regex_vec.iter().map(|log_event| log_event.1.regex).collect::<Vec<_>>();

        let regex_set = RegexSet::new(regex_slice.as_slice()).unwrap();

        return Parser {
            logger,
            regex_set,
            last_line: usize::MAX,
        };
    }
}

impl<'a> Base<'a> for Parser<'a> {
    fn get_lines_to_parse(self: &mut Self, log_path: &str) -> Option<Vec<String>> {
        let file = File::open(log_path).unwrap();
        let reader = BufReader::new(file);
        let lines: Vec<_> = reader.lines().collect();
        let line_count = lines.len();
        if self.last_line == usize::MAX {
            self.last_line = line_count;
        }
        if self.last_line == line_count {
            return None;
        }
        let line_diff = line_count - (line_count - self.last_line);
        let lines_to_parse = &lines[line_diff..];
        let results: Vec<String> = lines_to_parse.into_iter().map(|v| v.as_ref().unwrap().to_string()).collect();
        self.last_line = line_count;
    
        return Some(results);
    }

    fn parse(self: &'a Self, line: &'a str) -> Option<Log> {
        for matched_index in self.regex_set.matches(line) {
            match matched_index {
                _ => return capture_values(line, self.logger.log_events.get(&matched_index).unwrap()),
            }
        }

        return None;
    }

}

fn capture_values<'a>(line: &'a str, log_event: &LogEvent<'a>) -> Option<Log<'a>>  {
    let regex = Regex::new(log_event.regex).unwrap();
    if let Some(captures) = regex.captures(line) {
        return Some(Log {
            line,
            log_type: log_event.log_type,
            event_type: log_event.event_type,
            values: captures.iter().map(|v| String::from(v.unwrap().as_str())).collect::<Vec<String>>().drain(1..).collect(),
        });
    }

    return None;
}
