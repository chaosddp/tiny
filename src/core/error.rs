#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Internet connect error: {0}")]
    InternetConnectError(String),
    #[error("Invalid response from chat client: {0}")]
    ChatInvalidResponseError(String),
    #[error("Fail to call tool '{name}({id})', message: {message}")]
    ChatToolCallFailedError {
        name: String,
        id: String,
        message: String,
    },
    #[error("io error: {0:?}")]
    IoError(std::io::Error),
    #[error("lua error: {0:?}")]
    LuaError(mlua::error::Error),
    #[error("Runtime eror: {0}")]
    RuntimeError(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<mlua::Error> for Error {
    fn from(value: mlua::Error) -> Self {
        Error::LuaError(value)
    }
}
