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
