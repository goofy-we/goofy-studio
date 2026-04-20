use std::sync::Arc;

use super::AIProvider;

pub mod comfly;

pub use comfly::ComflyProvider;

pub const COMFLY_BASE_URL: &str = "https://ai.comfly.chat";

pub fn build_default_providers() -> Vec<Arc<dyn AIProvider>> {
    vec![Arc::new(ComflyProvider::new())]
}