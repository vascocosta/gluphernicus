use crate::cgi::Cgi;
use crate::log::{Category, Logger};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use urlencoding::decode;

#[derive(StructOpt)]
pub struct Config {
    #[structopt(short, long, default_value = ".", parse(from_os_str))]
    root: PathBuf,
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,
    #[structopt(short, long, default_value = "7070")]
    port: u32,
    #[structopt(short, long)]
    log: Option<String>,
}

pub struct Server {
    config: Config,
    logger: Arc<Mutex<Logger>>,
}

impl Server {
    pub async fn new(config: Config) -> io::Result<Self> {
        let logger = Arc::new(Mutex::new(
            Logger::new::<&str>(config.log.as_deref()).await?,
        ));

        Ok(Self { config, logger })
    }

    async fn handle_connection(&self, mut socket: TcpStream, address: &str) -> io::Result<()> {
        let mut buf = [0; 1024];

        match socket.read(&mut buf).await {
            Ok(0) => Ok(()),
            Ok(n) => {
                let request = String::from_utf8_lossy(&buf[1..n]);
                let response = self.handle_request(&request, address).await?;

                socket.write_all(response.as_slice()).await?;

                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    async fn handle_request(&self, request: &str, address: &str) -> io::Result<Vec<u8>> {
        let formatted_request =
            format!("{}/{}", self.config.root.to_string_lossy(), request.trim());
        let path = Path::new(&formatted_request);

        self.logger
            .lock()
            .await
            .log(
                Category::Request,
                format!("{} - /{}", address, request.trim()).as_str(),
            )
            .await?;

        if path.is_dir() {
            if path.join("gophermap").is_file() {
                let response = tokio::fs::read_to_string(path.join("gophermap")).await?;

                Ok(format!("{}.\r\n", response).into_bytes())
            } else {
                let menu = Menu::from_path(path, &self.config).await?;
                let response: String = menu
                    .items
                    .iter()
                    .map(|e| {
                        format!(
                            "{}{}\t/{}\t{}\t{}\r\n",
                            e.media,
                            e.description,
                            Menu::normalize_path(&e.selector),
                            e.host,
                            e.port
                        )
                    })
                    .collect();

                Ok(format!("{}.\r\n", response).into_bytes())
            }
        } else if decode(path.to_str().unwrap())
            .unwrap()
            .split('?')
            .take(1)
            .collect::<PathBuf>()
            .is_file()
        {
            if path
                .components()
                .any(|c| c.as_os_str().to_ascii_lowercase() == "cgi-bin")
            {
                let mut cgi = Cgi::new(path);
                let response = cgi.execute().await;

                Ok(response)
            } else {
                let response = tokio::fs::read(path).await?;

                Ok(response)
            }
        } else {
            let response = format!(
                "3 {} doesn't exist!\terror.host\t1\r\ni This resource cannot be located.\terror.host\t1",
                request.trim()
            );

            self.logger
                .lock()
                .await
                .log(
                    Category::Error,
                    format!("{} - /{} doesn't exist!", address, request.trim()).as_str(),
                )
                .await?;

            Ok(format!("{}\r\n.\r\n", response).into_bytes())
        }
    }

    pub async fn run(self: Arc<Self>) -> io::Result<()> {
        let listener =
            TcpListener::bind(format!("{}:{}", self.config.host, self.config.port)).await?;

        self.logger
            .lock()
            .await
            .log(
                Category::Info,
                format!("Listening on {}:{}", self.config.host, self.config.port).as_str(),
            )
            .await?;

        loop {
            let (socket, address) = listener.accept().await?;
            let self_clone = self.clone();

            tokio::spawn(async move {
                if let Err(error) = self_clone
                    .handle_connection(socket, &address.to_string())
                    .await
                {
                    eprintln!("Error handling the connection: {}", error);
                }
            });
        }
    }
}

struct Item {
    media: u32,
    description: String,
    selector: PathBuf,
    host: String,
    port: u32,
}

struct Menu {
    items: Vec<Item>,
}

impl Menu {
    #[allow(dead_code)]
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    async fn from_path(path: &Path, config: &Config) -> io::Result<Self> {
        if path.is_dir() {
            let mut items = Vec::new();
            let mut dir = tokio::fs::read_dir(path).await?;

            while let Some(dir_entry) = dir.next_entry().await? {
                let description = dir_entry.file_name().to_string_lossy().to_string();
                let selector = dir_entry
                    .path()
                    .strip_prefix(config.root.clone())
                    .unwrap_or(Path::new("/"))
                    .to_path_buf();

                items.push(Item {
                    media: if dir_entry.file_type().await?.is_dir() {
                        1
                    } else {
                        0
                    },
                    description,
                    selector,
                    host: config.host.clone(),
                    port: config.port,
                });
            }

            Ok(Self { items })
        } else {
            Ok(Self { items: Vec::new() })
        }
    }

    fn normalize_path<P: AsRef<Path>>(path: P) -> String {
        path.as_ref()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join("/")
    }
}
