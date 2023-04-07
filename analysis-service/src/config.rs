
use serde_yaml;

#[derive(Debug, serde::Deserialize, Copy, Clone)]
pub struct Config {
  pub max_depth: u32,
}

impl Config {
    pub fn read_config() -> Self {
        let fd = std::fs::File::open("sec_config.yaml").unwrap();
        let sec_config: Config = serde_yaml::from_reader(fd).unwrap();
        sec_config
    }
}