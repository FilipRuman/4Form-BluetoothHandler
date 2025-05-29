use chrono;
use colored::{self, ColoredString, Colorize};
use std::{fs::OpenOptions, io::Write, time};
pub enum LogPriority {
    Error,
    Info,
    Warning,
    Stage,
}
