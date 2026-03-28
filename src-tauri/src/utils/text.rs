//! 文本处理工具模块
//!
//! 提供字符串处理、格式转换等通用文本工具函数。

// ============================================================================
// ANSI 转义序列处理
// ============================================================================

/// 去除 ANSI 转义序列（颜色代码等）
///
/// 终端输出常包含 ANSI ESC 序列用于着色，此函数将其移除。
///
/// # 参数
/// * `input` - 包含 ANSI 转义序列的字符串
///
/// # 示例
/// ```
/// strip_ansi_codes("\x1b[32mHello\x1b[0m")
/// // -> "Hello"
pub fn strip_ansi_codes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// 从混合输出中提取 JSON 内容
///
/// 命令行工具输出常混有日志、进度条等文本，此函数提取其中的 JSON 部分。
///
/// # 参数
/// * `output` - 原始命令输出字符串
///
/// # 返回值
/// * `Some(String)` - 提取的 JSON 字符串
/// * `None` - 未找到有效的 JSON
///
/// # 识别规则
/// - JSON 对象以 `{` 开头
/// - JSON 数组以 `["`, `[{`, `[数字` 开头（非 `[plugins]` 等文本）
/// - 结尾以 `}` 或 `]` 结束
pub fn extract_json_from_output(output: &str) -> Option<String> {
    let clean_output = strip_ansi_codes(output);

    let lines: Vec<&str> = clean_output.lines().collect();
    let mut json_start_line = None;
    let mut json_end_line = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            json_start_line = Some(i);
            break;
        }
        if trimmed.starts_with('[') && trimmed.len() > 1 {
            let second_char = trimmed.chars().nth(1).unwrap_or(' ');
            if second_char == '"' || second_char == '{' || second_char == '[' || second_char.is_ascii_digit() {
                json_start_line = Some(i);
                break;
            }
        }
    }

    for (i, line) in lines.iter().enumerate().rev() {
        let trimmed = line.trim();
        if trimmed == "}" || trimmed == "}," || trimmed.ends_with('}') {
            json_end_line = Some(i);
            break;
        }
        if trimmed == "]" || trimmed == "]," {
            json_end_line = Some(i);
            break;
        }
    }

    match (json_start_line, json_end_line) {
        (Some(start), Some(end)) if start <= end => {
            let json_lines: Vec<&str> = lines[start..=end].to_vec();
            let json_str = json_lines.join("\n");
            Some(json_str)
        }
        _ => None,
    }
}

// ============================================================================
// 版本处理
// ============================================================================

/// 比较版本号，判断是否有更新版本
///
/// 支持 `v1.0.0` 或 `1.0.0` 格式，自动忽略前导 `v`。
///
/// # 参数
/// * `current` - 当前版本号
/// * `latest` - 最新版本号
///
/// # 返回值
/// * `true` - 有可用更新（latest > current）
/// * `false` - 无更新或版本相同
///
/// # 示例
/// ```
/// compare_versions("1.0.0", "1.0.1")  // -> true
/// compare_versions("v2.0.0", "1.9.9") // -> false
/// compare_versions("1.0.0", "1.0.0")  // -> false
pub fn compare_versions(current: &str, latest: &str) -> bool {
    let current = current.trim().trim_start_matches('v');
    let latest = latest.trim().trim_start_matches('v');

    let current_parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let latest_parts: Vec<u32> = latest
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    for i in 0..3 {
        let c = current_parts.get(i).unwrap_or(&0);
        let l = latest_parts.get(i).unwrap_or(&0);
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    false
}

// ============================================================================
// 字符串脱敏
// ============================================================================

/// 对敏感字符串进行脱敏处理
///
/// 出于安全考虑，日志中不应明文显示敏感信息。
///
/// # 规则
/// - 长度 <= 8: 显示 `****`
/// - 长度 9-16: 显示前4字符 + `****`
/// - 长度 > 16: 显示前4字符 + `****` + 后4字符
///
/// # 示例
/// ```
/// mask_sensitive_string("password123")  // -> "pass****3123"
/// mask_sensitive_string("secret")       // -> "****"
pub fn mask_sensitive_string(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let len = s.len();
    if len <= 8 {
        "****".to_string()
    } else if len <= 16 {
        format!("{}****", &s[..4])
    } else {
        format!("{}****{}", &s[..4], &s[len - 4..])
    }
}

// ============================================================================
// ID 标准化
// ============================================================================

/// 标准化智能体标识符
///
/// 将智能体名称转换为合法的目录/文件名字符串：
/// - 转换为小写
/// - 将非字母数字字符替换为 `-`
/// - 去除首尾的 `-`
///
/// # 参数
/// * `name` - 原始智能体名称
///
/// # 示例
/// ```
/// normalize_agent_id("My Agent 123")
/// // -> "my-agent-123"
pub fn normalize_agent_id(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

// ============================================================================
// 时间工具
// ============================================================================

/// 获取当前时间戳（毫秒）
///
/// 计算自 Unix 纪元（1970-01-01）以来的毫秒数。
///
/// # 返回值
/// * `u64` - 当前时间戳（毫秒）
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
