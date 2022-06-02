//! Templates

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde::Serialize;
use walkdir::WalkDir;

use crate::config::Config;

pub mod blog;

/// Returns the files for all templates
///
/// # Returns
///
/// The returned HashMap contains the template name as the key,
/// and an object with the trait RustEmbed which allows to retrive the files.
pub fn get_templates_files() -> HashMap<String, HashMap<String, Vec<u8>>> {
    const TEMPLATES: [&'static str; 1] = ["blog"];

    let mut m: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();
    for tpl_name in TEMPLATES {
        match tpl_name {
            "blog" => {
                let mut files: HashMap<String, Vec<u8>> = HashMap::new();
                for asset in blog::Assets::iter() {
                    let name = asset.to_string();
                    // eprintln!("Retrieving file {name}");
                    let file = blog::Assets::get(name.as_str()).expect("Invalid template file");
                    files.insert(name, file.data.into_owned());
                }
                m.insert(tpl_name.to_string(), files);
            }
            // NB: add other templates here
            _ => {
                panic!("Unknown template: {}", tpl_name);
            }
        }
    }
    m
}

/// Index template data
#[derive(Debug, Clone, Serialize)]
pub struct IndexTemplateData {
    /// Page HTML content
    pub content: String,
}

/// Page template data
#[derive(Debug, Clone, Serialize)]
pub struct PageTemplateData {
    /// Page HTML content
    pub content: String,
}

/// Templates registry for a single template
///
/// The template registry contains 2 templates:
/// - "index"
/// - "page"
#[derive(Debug, Clone)]
pub struct TemplatesRegistry {
    /// Template directory inside the config folder
    config_template_dir: PathBuf,
    /// Handlebars registry
    registry: Handlebars<'static>,
}

impl TemplatesRegistry {
    /// Initializes the template registry
    pub fn init(config: &Config) -> Result<Self> {
        let config_template_dir = config.template_dir();

        // find index.hbs and pahe.hbs
        let index_hbs_str = fs::read_to_string(config_template_dir.join("index.hbs"))?;
        let page_hbs_str = fs::read_to_string(config_template_dir.join("page.hbs"))?;

        // setup Handlebars registry
        let mut registry = Handlebars::new();
        registry
            .register_template_string("index", &index_hbs_str)
            .context("Invalid template")?;
        registry
            .register_template_string("page", &page_hbs_str)
            .context("Invalid template")?;

        Ok(TemplatesRegistry {
            config_template_dir,
            registry,
        })
    }

    /// Reloads the registry templates
    pub fn reload_templates(&mut self) -> Result<()> {
        let index_hbs_str = fs::read_to_string(self.config_template_dir.join("index.hbs"))?;
        let page_hbs_str = fs::read_to_string(self.config_template_dir.join("page.hbs"))?;

        self.registry
            .register_template_string("index", &index_hbs_str)
            .context("Invalid template")?;
        self.registry
            .register_template_string("page", &page_hbs_str)
            .context("Invalid template")?;
        Ok(())
    }

    /// Renders the template for the index
    pub fn render_index(&self, data: &IndexTemplateData) -> Result<String> {
        let rendered = self.registry.render("index", data)?;
        Ok(rendered)
    }

    /// Renders the template for the page
    pub fn render_page(&self, data: &PageTemplateData) -> Result<String> {
        let rendered = self
            .registry
            .render("page", data)
            .context("Cannot render")?;
        Ok(rendered)
    }

    /// Copies the template static assets to the target directory
    ///
    /// # Notes
    ///
    /// .hbs files are excluded
    pub fn copy_static_assets<P: AsRef<Path>>(&self, to: P) -> Result<()> {
        let src_dir = &self.config_template_dir;
        for res_entry in WalkDir::new(&src_dir) {
            let entry = res_entry.unwrap();
            if entry.path().is_dir() || entry.path().extension().unwrap() == "hbs" {
                continue;
            }
            let entry_path_short = entry.path().strip_prefix(&src_dir).unwrap();
            let mut target_file = PathBuf::new();
            target_file.push(&to);
            let target_file = target_file.join(entry_path_short);
            fs::copy(entry.path(), target_file).context("Failed to copy")?;
        }

        Ok(())
    }
}
