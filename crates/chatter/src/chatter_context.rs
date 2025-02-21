use async_openai::types::ChatCompletionRequestMessage;

#[derive(Clone, Default)]
pub struct ChatterContext {
    pub messages: Vec<ChatCompletionRequestMessage>,
}
