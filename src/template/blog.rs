//! Blog template

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/template/blog/"]
pub struct Assets;
