use chrono;
use colored::{self, ColoredString, Colorize};
use std::{fs::OpenOptions, io::Write, time};
pub enum LogPriority {
    Error,
    Info,
    Warning,
    Stage,
}
impl LogPriority {
    pub fn markdown_formatting(&self, text: &String) -> String {
        let time = chrono::offset::Local::now().format("%d-%b %H:%M");
        match self {
            LogPriority::Error => format!("# {} [Err]: {}", time, text),
            LogPriority::Info => format!("### {} [Inf]: {}", time, text),
            LogPriority::Warning => format!("## {} [Warn]: {}", time, text),
            LogPriority::Stage => format!("\n#### {} [Stg]: {}\n", time, text),
        }
    }
    pub fn format_println_text(&self, text: &String) -> ColoredString {
        match self {
            LogPriority::Error => format!("Err]: {text} ").red(),
            LogPriority::Info => format!("[Inf]: {text} ").blue(),
            LogPriority::Warning => format!("[Warn]: {text} ").yellow(),
            LogPriority::Stage => format!("\n [Stg]: {text} \n").green(),
        }
    }
}
pub const LOG_FILE_PATH: &str = "./../Logs.md";
pub const SHOW_ERRORS_THRU_PRINTLN: bool = true;

// this is not the most efficient but clean!
pub fn default_log(value: String, priority: LogPriority) {
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

    file.write_all((text + "\n").as_bytes()).unwrap();
}
