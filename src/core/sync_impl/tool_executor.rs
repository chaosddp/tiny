use crate::core::TinyResult;

#[derive(Debug)]
pub struct ToolCallResult {
    pub name: String,
    pub id: String,
    pub result: String,
}

pub trait ToolExecutor {
    fn execute(&self, name: &str, id: &str, arguments: &str) -> TinyResult<ToolCallResult>;
}
