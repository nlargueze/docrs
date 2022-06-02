//! Build system

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use walkdir::WalkDir;

use crate::{
    render::HtmlRenderer,
    template::{IndexTemplateData, PageTemplateData, TemplatesRegistry},
};

/// PageBuilder builds a single
#[derive(Debug, Clone)]
pub struct PageBuilder {
    /// Source directory
    pub src_dir: PathBuf,
    /// Build directory
    pub build_dir: PathBuf,
    /// Template files directory
    pub template_dir: PathBuf,
    /// Renderer
    pub renderer: HtmlRenderer,
    /// Registry
    pub registry: TemplatesRegistry,
}

impl PageBuilder {
    /// Builds the site
    pub fn build_all(&self) -> Result<()> {
        // clear + create build dir
        let build_dir = &self.build_dir;
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir)?;
        }
        fs::create_dir(&build_dir)?;

        // copy template assets to the build folder
        let registry = &self.registry;
        self.registry.copy_static_assets(&build_dir)?;

        // build index.html
        let index_str = registry.render_index(&IndexTemplateData {
            content: "".to_string(),
        })?;
        fs::write(build_dir.join("index.html"), &index_str)?;

        // build pages
        let src_dir = &self.src_dir;
        for res_entry in WalkDir::new(&src_dir) {
            let entry = res_entry.unwrap();
            let entry_path = entry.path();
            if entry_path.is_dir() {
                continue;
            }
            // println!("{}", entry_path.display());

            self.build_page(entry_path)?;
        }

        Ok(())
    }

    /// Builds a single page
    pub fn build_page<P: AsRef<Path>>(&self, page: P) -> Result<()> {
        // MD --> HTML
        let renderer = &self.renderer;
        let html = renderer.render(page.as_ref())?;

        // HTML ~~> TEMPLATE
        let registry = &self.registry;
        let content = registry.render_page(&PageTemplateData { content: html })?;

        // write file
        let target_page = self.get_target_page(page.as_ref())?;
        if let Some(parent) = target_page.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target_page, content)?;

        Ok(())
    }

    /// Removes a source page from the build folder
    pub fn remove_page<P: AsRef<Path>>(&self, page: P) -> Result<()> {
        let target_page = self.get_target_page(page.as_ref())?;
        fs::remove_file(target_page)?;
        Ok(())
    }

    /// Renames a page in the target folder
    pub fn rename_page<P: AsRef<Path>>(&self, from: P, to: P) -> Result<()> {
        let target_from = self.get_target_page(from.as_ref())?;
        let target_to = self.get_target_page(to.as_ref())?;
        fs::rename(target_from, target_to)?;
        Ok(())
    }

    /// Returns the destination path for a source page
    fn get_target_page<P: AsRef<Path>>(&self, page: P) -> Result<PathBuf> {
        let src_dir = &self.src_dir;
        let build_dir = &self.build_dir;
        let page_stripped = page.as_ref().strip_prefix(&src_dir).unwrap();
        let mut page_target = build_dir.join(&page_stripped);
        page_target.set_extension("html");
        Ok(page_target.to_owned())
    }
}
