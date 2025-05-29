use chrono;
use colored::{self, ColoredString, Colorize};
use std::{fs::OpenOptions, io::Write, time};

#[derive(PartialEq)]
pub enum LogPriority {
    Error,
    Info,
    Warning,
    Stage,
}
impl LogPriority {
    pub fn markdown_formatting(&self, text: &str) -> String {
        let time = chrono::offset::Local::now().format("%d %H:%M");
        match self {
            LogPriority::Error => {
                format!("# {} [Err]: {}\n", time, text)
            }
            LogPriority::Info => {
                format!("### {} [Inf]: {}\n", time, text)
            }
            LogPriority::Warning => {
                format!("## {} [Warn]: {}\n", time, text)
            }
            LogPriority::Stage => {
                format!("\n#### {} [Stg]: {}\n\n", time, text)
            }
        }
    }
    pub fn format_println_text(&self, text: &str) -> ColoredString {
        match self {
            LogPriority::Error => format!("[Err]: {}", text).red(),
            LogPriority::Info => format!("[Inf]: {}", text).blue(),
            LogPriority::Warning => format!("[Warn]: {}", text).yellow(),
            LogPriority::Stage => format!("[Stg]: {}", text).green(),
        }
    }
}
pub const LOG_FILE_PATH: &str = "./Logs.md";
pub const SHOW_ERRORS_THRU_PRINTLN: bool = true;
// hashset would be faster but I'd have to use constant one from crate
pub const LOG_LEVEL: [LogPriority; 4] = [
    LogPriority::Warning,
    LogPriority::Stage,
    LogPriority::Info,
    LogPriority::Error,
];
// this is not the most efficient but clean!
pub fn default_log(value: &str, priority: LogPriority) {
    if !LOG_LEVEL.contains(&priority) {
        return;
    }
    if SHOW_ERRORS_THRU_PRINTLN {
        println!("{}", priority.format_println_text(&value));
    }
    save_text_to_log_file(priority.markdown_formatting(&value));
}

fn save_text_to_log_file(text: String) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&LOG_FILE_PATH)
        .expect(&format!(
            "path for logging file:{} was not correct!",
            &LOG_FILE_PATH
        ));

    file.write_all((text).as_bytes()).unwrap();
}
