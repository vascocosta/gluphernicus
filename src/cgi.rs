use std::{collections::HashMap, path::Path};
use tokio::process::Command;
use urlencoding::decode;

pub struct Cgi {
    script: Command,
}

// Extremely experimental code that needs a lot of work.

impl Cgi {
    pub fn new(script: &Path) -> Self {
        let script = decode(script.to_str().unwrap()).unwrap();
        let query_string: String = script.split('?').skip(1).collect();
        let mut envs: HashMap<String, String> = HashMap::new();
        let mut command = Command::new(script.split('?').take(1).collect::<String>());

        envs.insert(String::from("QUERY_STRING"), query_string);
        command.envs(envs);

        Self { script: command }
    }

    pub async fn execute(&mut self) -> Vec<u8> {
        let output = self.script.output().await.unwrap();

        output.stdout
    }
}
