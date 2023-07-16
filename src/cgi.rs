use std::{collections::HashMap, io, path::Path};
use tokio::process::Command;
use urlencoding::decode;

pub struct Cgi {
    script: Command,
}

impl Cgi {
    pub fn new(script: &Path) -> Option<Self> {
        let script = decode(script.to_str()?).ok()?;
        let query_string: String = script.split('?').skip(1).collect();
        let mut envs: HashMap<String, String> = HashMap::new();
        let mut command = Command::new(script.split('?').take(1).collect::<String>());

        envs.insert(String::from("QUERY_STRING"), query_string);
        command.envs(envs);

        Some(Self { script: command })
    }

    pub async fn execute(&mut self) -> io::Result<Vec<u8>> {
        let output = self.script.output().await?;

        Ok(output.stdout)
    }
}
