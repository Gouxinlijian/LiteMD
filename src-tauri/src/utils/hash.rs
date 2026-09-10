use md5::{Digest, Md5};

/// 计算字符串内容的 MD5 十六进制，用于进度恢复时判断文件是否被外部修改
pub fn md5_hex(content: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
