use std::sync::Arc;

use crate::chatter_context::ChatterContext;
use derive_builder::Builder;

#[derive(Builder, Clone, Default)]
pub struct Chatter {
    pub context: Arc<ChatterContext>,
    pub client: async_openai::Client<async_openai::config::OpenAIConfig>,
}
