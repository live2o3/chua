use serde::{Deserialize, Serialize};
use std::ops::Range;
use uuid::Uuid;

macro_rules! impl_from_error {
    ($st:ident) => {
        impl<E: std::error::Error> From<E> for $st {
            fn from(e: E) -> Self {
                Self::Other {
                    detail: e.to_string(),
                }
            }
        }
    };
}

/// 初始化请求的参数
#[derive(Serialize, Deserialize, Debug)]
pub struct UploadParam {
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
#[serde(tag = "result")]
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
#[serde(tag = "type")]
pub enum InitializeError {
    /// 文件尺寸错误
    Size { max: u64 },

    /// 分片大小不合适，并给出建议的分片大小
    ChunkSize { max: u64 },

    /// 其它错误
    Other { detail: String },
}

impl_from_error!(InitializeError);

/// 分片上传响应的结果
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "result")]
pub enum UploadChunkResult {
    Ok,
    Err { error: UploadChunkError },
}

/// 分片上传响应的错误
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum UploadChunkError {
    /// 这个分片的尺寸不对
    Size { expected: u64, actual: u64 },

    /// 其它错误
    Other { detail: String },
}

impl_from_error!(UploadChunkError);

/// 完成响应的结果
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "result")]
pub enum CompleteResult {
    Ok,
    Err { error: CompleteError },
}

/// 完成响应的错误
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum CompleteError {
    /// 没上传完
    Incomplete {
        param: UploadParam,
        ranges: Vec<Range<usize>>,
    },

    /// MD5 校验不合法
    MD5 { expected: String, actual: String },

    /// 其它错误
    Other { detail: String },
}

impl_from_error!(CompleteError);
