use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

struct Config {
    root: PathBuf,
    host: String,
    port: u32,
}

struct Server {
    config: Config,
}

impl Server {
    fn new(config: Config) -> Self {
        Self { config }
    }

    async fn handle_connection(&self, mut socket: TcpStream) -> io::Result<()> {
        let mut buf = [0; 1024];

        match socket.read(&mut buf).await {
            Ok(0) => Ok(()),
            Ok(n) => {
                let request = String::from_utf8_lossy(&buf[1..n]);
                let response = self.handle_request(&request)?;

                socket.write_all(response.as_slice()).await?;

                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn handle_request(&self, request: &str) -> io::Result<Vec<u8>> {
        let formatted_request =
            format!("{}/{}", self.config.root.to_string_lossy(), request.trim());
        let path = Path::new(&formatted_request);

        if path.is_dir() {
            if path.join("gophermap").is_file() {
                let response = fs::read(path.join("gophermap"))?;

                return Ok(response);
            }

            let menu = Menu::from_path(path, &self.config);
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

            Ok(format!("{}.\r\n", response).into())
        } else {
            let response = fs::read(path)?;

            Ok(response)
        }
    }

    async fn run(self: Arc<Self>) -> io::Result<()> {
        let listener =
            TcpListener::bind(format!("{}:{}", self.config.host, self.config.port)).await?;

        loop {
            let (socket, _) = listener.accept().await?;
            let self_clone = self.clone();

            tokio::spawn(async move {
                if let Err(error) = self_clone.handle_connection(socket).await {
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

    fn from_path(path: &Path, config: &Config) -> Self {
        if path.is_dir() {
            let items = fs::read_dir(path).unwrap();

            let items: Vec<Item> = items
                .map(|f| {
                    let dir_entry = f.unwrap();
                    let description = dir_entry.file_name().to_string_lossy().to_string();
                    let selector = dir_entry
                        .path()
                        .strip_prefix(config.root.clone())
                        .unwrap()
                        .to_path_buf();

                    Item {
                        media: if dir_entry.file_type().unwrap().is_dir() {
                            1
                        } else {
                            0
                        },
                        description,
                        selector,
                        host: config.host.clone(),
                        port: config.port,
                    }
                })
                .collect();

            Self { items }
        } else {
            Self { items: Vec::new() }
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

#[tokio::main]
async fn main() {
    let config = Config {
        root: PathBuf::from("."),
        host: String::from("127.0.0.1"),
        port: 7070,
    };

    let server = Arc::new(Server::new(config));

    if let Err(error) = server.run().await {
        eprintln!("Error running server: {}", error);
    }
}
