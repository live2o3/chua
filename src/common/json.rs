use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 初始化请求的参数
#[derive(Serialize, Deserialize, Debug)]
pub struct InitializeParam {
    /// 文件大小
    pub size: u64,

    /// 分片大小
    pub chunk_size: u64,

    /// 扩展名
    pub extension: String,

    /// md5
    pub md5: String,
}

/// 初始化响应的结果
#[derive(Serialize, Deserialize, Debug)]
pub enum InitializeResult {
    /// 成功
    Ok {
        /// 文件ID
        id: Uuid,

        /// 是否已上传过
        duplicated: bool,
    },
    Err {
        /// 错误
        error: InitializeError,
    },
}

/// 初始化响应的错误
#[derive(Serialize, Deserialize, Debug)]
pub enum InitializeError {
    /// 文件尺寸错误
    Size,

    /// 分片大小不合适，并给出建议的分片大小
    ChunkSize(u64),

    /// 其它错误
    Other(String),
}

/// 完成请求的参数
#[derive(Serialize, Deserialize, Debug)]
pub struct CompleteParam {
    /// 文件ID
    pub id: Uuid,
}

/// 完成响应的结果
#[derive(Serialize, Deserialize, Debug)]
pub enum CompleteResult {
    Ok,
    Err { error: CompleteError },
}

/// 完成响应的错误
#[derive(Serialize, Deserialize, Debug)]
pub enum CompleteError {
    Uploading,
    MD5(String),
}
