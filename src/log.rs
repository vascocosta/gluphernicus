use chrono::Utc;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub enum Category {
    Error,
    Info,
    Request,
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Error => write!(f, "Error"),
            Category::Info => write!(f, "Info"),
            Category::Request => write!(f, "Request"),
        }
    }
}

pub struct Logger {
    output: Box<dyn Send + Write>,
}

impl Logger {
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> io::Result<Self> {
        let output: Box<dyn Send + Write> = match path {
            Some(path) => Box::new(OpenOptions::new().append(true).create(true).open(path)?),
            None => Box::new(io::stdout()),
        };

        Ok(Self { output })
    }

    pub fn log(&mut self, category: Category, message: &str) -> io::Result<()> {
        writeln!(self.output, "[{}] {}: {}", Utc::now(), category, message)?;
        self.output.flush()
    }
}
