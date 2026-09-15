use async_trait::async_trait;

use crate::core::aloop::loop_base::{LoopError, ToolExecutor};

pub struct DefaultToolExecutor {}

impl DefaultToolExecutor {
    pub fn new() -> Self {
        DefaultToolExecutor {  }
    }
}

#[async_trait]
impl ToolExecutor for DefaultToolExecutor {
    async fn exec(
        &self,
        name: &str,
        id: &str,
        arguments: Option<&str>,
    ) -> Result<String, LoopError> {
        Ok("result".to_string())
    }
}
