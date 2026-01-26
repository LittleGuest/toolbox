//! 全局异常

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    E(String),
    #[error("序列化错误")]
    SerializeError,
    #[error("验证码错误")]
    CaptchaError,
    #[error("验证码失效")]
    CaptchaExpireError,
    #[error("账号或密码错误")]
    UsernameOrPasswordError,
    #[error("用户不存在")]
    UserNotFound,
    #[error("{0}")]
    JwtError(&'static str),
    #[error("服务器异常 : {0}")]
    ServerError(&'static str),
    #[error("SQL错误")]
    SqlError,
    #[error("参数校验错误: {0}")]
    ValidationError(String),
    #[error("未知错误")]
    Unknown,

    #[error(transparent)]
    PoemError(#[from] poem::Error),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RequestErr(#[from] reqwest::Error),
    #[error(transparent)]
    SqlxErr(#[from] sqlx::Error),
    #[error(transparent)]
    TeraErr(#[from] tera::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;


