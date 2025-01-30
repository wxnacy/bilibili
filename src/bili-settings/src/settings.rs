use std::{env, path::{Path, PathBuf}};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Part {
    pub home: String,
    pub names: Vec<String>,
}

impl Part {
    pub fn home(&self) -> PathBuf {
        PathBuf::from(&self.home)
    }

    pub fn get_path(&self, name: &str, id: &str) -> PathBuf {
        self.home().join(name).join(format!("{}.ts", id))
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct App {
    // 多媒体目录
    pub media_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Up {
    // 多媒体目录
    pub mid: u64,
    pub name: String,
    pub default: bool,
}

impl Up {
    pub fn get_cookie_path(&self) -> PathBuf {
        let name = format!("{}.json", &self.mid);
        Settings::cookie().join(name)
    }
}


#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub app: App,
    pub part: Part,
    pub up: Vec<Up>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Self::new_config()?.try_deserialize()
    }

    pub fn new_config() -> Result<Config, ConfigError> {
        let config_home = Self::home();
        let config_path = config_home.join("bilibili.toml");

        Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::from(config_path))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("BILI"))
            .build()

        // Now that we're done, let's access our configuration
        // println!("debug: {:?}", s.get_bool("debug"));
        // println!("database: {:?}", s.get::<String>("database.url"));

        // // You can deserialize (and thus freeze) the entire configuration as
        // s.try_deserialize()
    }

    pub fn build_config<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
        Config::builder()
            .add_source(File::from(path.as_ref()))
            .build()
    }

    pub fn home() -> PathBuf {
        let config_home = env::var("BILI_CONFIG_HOME").unwrap_or_else(|_| "~/.bilibili".into());
        lazytool::expand_user(config_home)
    }

    pub fn cache() -> PathBuf {
        Self::home().join("cache")
    }

    pub fn cookie() -> PathBuf {
        Self::home().join("cookie")
    }

    pub fn media() -> PathBuf {
        Self::home().join("media")
    }

    pub fn part() -> PathBuf {
        Self::home().join("part.json")
    }

    pub fn get_default_up(&self) -> Option<&Up> {
        self.up.iter().filter(|x| x.default).next()
    }

    pub fn get_up(&self, mid: Option<u64>) -> Option<&Up> {
        if let Some(id) = mid {
            return self.up.iter().filter(|x| x.mid == id).next();
        }
        self.get_default_up()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_new() {
        todo!();
    }
}
