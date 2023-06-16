use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const ROOT: &str = ".";
const HOST: &str = "127.0.0.1";
const PORT: u32 = 7070;

#[derive(Clone)]
struct Server {
    root: PathBuf,
    host: String,
    port: u32,
}

impl Server {
    fn new(root: Option<&str>, host: Option<&str>, port: Option<u32>) -> Self {
        Self {
            root: root.unwrap_or(".").into(),
            host: String::from(host.unwrap_or("127.0.0.1")),
            port: port.unwrap_or(70),
        }
    }

    async fn handle_connection(mut socket: TcpStream) {
        let mut buf = [0; 1024];

        match socket.read(&mut buf).await {
            Ok(0) => (),
            Ok(n) => {
                let request = String::from_utf8_lossy(&buf[1..n]);
                let response = handle_request(&request);
                socket
                    .write_all(response.unwrap().as_slice())
                    .await
                    .unwrap();
            }
            Err(_) => (),
        }
    }

    async fn run(&self) -> io::Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).await?;

        loop {
            let (socket, _) = listener.accept().await?;

            tokio::spawn(async move {
                Self::handle_connection(socket).await;
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
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn from_path(path: &Path) -> Self {
        if path.is_dir() {
            let items = fs::read_dir(path).unwrap();

            let items: Vec<Item> = items
                .map(|f| {
                    let dir_entry = f.unwrap();
                    let description = dir_entry.file_name().to_string_lossy().to_string();
                    let selector = dir_entry.path().strip_prefix(ROOT).unwrap().to_path_buf();

                    Item {
                        media: if dir_entry.file_type().unwrap().is_dir() {
                            1
                        } else {
                            0
                        },
                        description,
                        selector,
                        host: String::from(HOST),
                        port: PORT,
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

fn handle_request(request: &str) -> io::Result<Vec<u8>> {
    let formatted_request = format!("{ROOT}/{}", request.trim());
    let path = Path::new(&formatted_request);

    if path.is_dir() {
        let menu = Menu::from_path(path);
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

#[tokio::main]
async fn main() {
    let server = Server::new(None, None, Some(7070));

    if let Err(error) = server.run().await {
        eprintln!("{error}");
    }
}
