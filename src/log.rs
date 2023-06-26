use chrono::Utc;
use std::fmt::Display;
use std::io;
use std::path::Path;
use std::pin::Pin;
use tokio::io::{AsyncWrite, AsyncWriteExt};

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
    output: Pin<Box<dyn Send + AsyncWrite>>,
}

impl Logger {
    pub async fn new<P: AsRef<Path>>(path: Option<P>) -> io::Result<Self> {
        let output: Pin<Box<dyn Send + AsyncWrite>> = match path {
            Some(path) => Box::pin(
                tokio::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path)
                    .await?,
            ),
            None => Box::pin(tokio::io::stdout()),
        };

        Ok(Self { output })
    }

    pub async fn log(&mut self, category: Category, message: &str) -> io::Result<()> {
        self.output
            .write_all(
                format!(
                    "[{}] {}: {}\n",
                    Utc::now().format("%d/%m/%Y %H:%M:%S%.3f"),
                    category,
                    message
                )
                .as_bytes(),
            )
            .await?;

        self.output.flush().await
    }
}
