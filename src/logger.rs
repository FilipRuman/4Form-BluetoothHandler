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
            LogPriority::Error => {
                if LOG_ERROR {
                    format!("# {} [Err]: {}\n", time, text)
                } else {
                    String::new()
                }
            }
            LogPriority::Info => {
                if LOG_INFO {
                    format!("### {} [Inf]: {}\n", time, text)
                } else {
                    String::new()
                }
            }
            LogPriority::Warning => {
                if LOG_WARNING {
                    format!("## {} [Warn]: {}\n", time, text)
                } else {
                    String::new()
                }
            }
            LogPriority::Stage => {
                if LOG_STAGE {
                    format!("\n#### {} [Stg]: {}\n\n", time, text)
                } else {
                    String::new()
                }
            }
        }
    }
    pub fn format_println_text(&self, text: &String) -> ColoredString {
        match self {
            LogPriority::Error => {
                if LOG_ERROR {
                    format!("[Err]: {}", text).red()
                } else {
                    "".clear()
                }
            }
            LogPriority::Info => {
                if LOG_INFO {
                    format!("[Inf]: {}", text).blue()
                } else {
                    "".clear()
                }
            }
            LogPriority::Warning => {
                if LOG_WARNING {
                    format!("[Warn]: {}", text).yellow()
                } else {
                    "".clear()
                }
            }
            LogPriority::Stage => {
                if LOG_STAGE {
                    format!("[Stg]: {}", text).green()
                } else {
                    "".clear()
                }
            }
        }
    }
}
pub const LOG_FILE_PATH: &str = "./../Logs.md";
pub const SHOW_ERRORS_THRU_PRINTLN: bool = true;
// log levels
pub const LOG_INFO: bool = true;
pub const LOG_WARNING: bool = true;
pub const LOG_STAGE: bool = true;
pub const LOG_ERROR: bool = true;

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

    file.write_all((text).as_bytes()).unwrap();
}
