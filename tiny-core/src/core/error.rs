/// Error from this application
#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error when open file: {0}")]
    IoError(std::io::Error),
    #[error("Fail to call lua content: {0}")]
    LuaError(mlua::Error),
    #[error("Request error with status code: {0}, message: {1}")]
    HttpError(u16, String),
    #[error("Lua reference droped.")]
    InvalidLuaReference,
    #[error("Invalid json: {0}")]
    InvalidJson(String),
    #[error("runtime error: {0}")]
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
