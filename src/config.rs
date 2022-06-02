//! Configuration

use std::{env, fs, path::PathBuf, str::FromStr};

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};

use crate::template::get_templates_files;

/// Config directory name
const CONFIG_DIR_NAME: &str = ".docrs";

/// Config file name
const CONFIG_FILE_NAME: &str = "config.toml";

/// Template Options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOptions {
    /// Template name
    pub name: String,
}

impl Default for TemplateOptions {
    fn default() -> Self {
        Self {
            name: "blog".to_string(),
        }
    }
}

/// Page configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    /// Page name
    pub title: String,
    /// Page path
    pub path: String,
    /// Page subpages
    pub pages: Option<Vec<PageConfig>>,
}

/// Configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// Template options
    pub template: TemplateOptions,
    /// Index configuration
    pub index: Vec<PageConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            template: TemplateOptions::default(),
            index: vec![PageConfig {
                title: "Hello World".to_string(),
                path: "./hello.md".to_string(),
                pages: None,
            }],
        }
    }
}

impl Config {
    /// Loads the configuration from the file system
    pub fn load() -> Result<Self> {
        let config = Config::default();
        let file = config.config_dir().join(CONFIG_FILE_NAME);
        let file_str = fs::read_to_string(file).context("Invalid config file")?;
        let config = toml::from_str::<Config>(&file_str).context("Invalid config")?;
        Ok(config)
    }

    /// Returns the CWD
    pub fn cwd(&self) -> PathBuf {
        cwd()
    }

    /// Returns the config folder
    pub fn config_dir(&self) -> PathBuf {
        env::current_dir().unwrap().join(CONFIG_DIR_NAME)
    }

    /// Returns the temporary build folder
    pub fn tmp_build_dir(&self) -> PathBuf {
        self.config_dir().join("build")
    }

    /// Returns the src folder
    pub fn src_dir(&self) -> PathBuf {
        cwd().join("src")
    }

    /// Returns the build folder
    pub fn build_dir(&self) -> PathBuf {
        cwd().join("build")
    }

    /// Returns the template folder
    pub fn templates_dir(&self) -> PathBuf {
        self.config_dir().join("templates")
    }

    /// Returns the template folder
    pub fn template_dir(&self) -> PathBuf {
        self.templates_dir().join(&self.template.name)
    }

    /// Checks if the repo has been initialized
    pub fn is_initialized(&self) -> bool {
        self.config_dir().exists()
    }

    /// Initializes the repo
    pub fn init_repo(&self) -> Result<()> {
        let config_dir = self.config_dir();
        fs::create_dir(&config_dir).context("Cannot create config folder")?;

        let config_str = toml::to_string(self).context("Cannot generate config")?;
        fs::write(&config_dir.join(CONFIG_FILE_NAME), &config_str)
            .context("Cannot write config")?;

        // copy templates files
        let tpl_files = get_templates_files();
        for (name, files) in tpl_files.iter() {
            let tpl_dir = self.templates_dir().join(name);
            for (file_name, file_content) in files {
                let file_path = tpl_dir.join(file_name);
                let file_dir = file_path.parent().unwrap();
                fs::create_dir_all(&file_dir).context("Cannot create folder")?;
                fs::write(&file_path, file_content).context("Cannot write config template file")?;
            }
        }

        // init dummy src folder
        let src_dir = self.src_dir();
        if !src_dir.exists() {
            fs::create_dir(&src_dir).context("Cannot create src folder")?;
            let hello_file_str = "# Hello World";
            let hello_file = &src_dir.join("hello.md");
            fs::write(hello_file, hello_file_str).context("Cannot write src/index.md file")?;
        }

        // .gitignore
        fs::write(cwd().join(".gitignore"), "/build\n/.docrs/build")
            .context("Cannot write .gitignore")?;

        Ok(())
    }

    /// Deletes the config folder
    pub fn reset_repo(&self) -> Result<()> {
        let config_dir = self.config_dir();
        fs::remove_dir_all(config_dir).context("Cannot delete config")?;
        Ok(())
    }
}

/// Sets the current working directory
pub fn set_current_dir(wd: &Option<String>) -> Result<()> {
    if let Some(x) = wd {
        let cwd = PathBuf::from_str(x).context("Invalid path")?;
        if !cwd.is_dir() {
            return Err(Error::msg("Not a directory"));
        }
        env::set_current_dir(cwd)?;
    };
    Ok(())
}

/// Returns the current working directory
fn cwd() -> PathBuf {
    env::current_dir().unwrap()
}
