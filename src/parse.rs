use anyhow::Result;
use memchr::memchr_iter;
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

pub fn mmap_file(path: &Path) -> Result<Mmap> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(mmap)
}

pub fn find_line_ranges(data: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::with_capacity(data.len() / 500);
    let mut start = 0;

    for pos in memchr_iter(b'\n', data) {
        if pos > start {
            ranges.push((start, pos));
        }
        start = pos + 1;
    }

    if start < data.len() {
        ranges.push((start, data.len()));
    }

    ranges
}

pub fn parse_jsonl_line(line: &[u8]) -> Option<serde_json::Value> {
    let trimmed = trim_whitespace(line);
    if trimmed.is_empty() {
        return None;
    }
    let mut owned = trimmed.to_vec();
    simd_json::serde::from_slice(&mut owned).ok()
}

fn trim_whitespace(data: &[u8]) -> &[u8] {
    let start = data.iter().position(|&b| !b.is_ascii_whitespace()).unwrap_or(data.len());
    let end = data.iter().rposition(|&b| !b.is_ascii_whitespace()).map_or(start, |p| p + 1);
    &data[start..end]
}

pub fn search_json_value(value: &serde_json::Value, query_lower: &str) -> bool {
    match value {
        serde_json::Value::String(s) => s.to_lowercase().contains(query_lower),
        serde_json::Value::Array(arr) => arr.iter().any(|item| search_json_value(item, query_lower)),
        serde_json::Value::Object(obj) => obj.values().any(|v| search_json_value(v, query_lower)),
        _ => false,
    }
}

pub fn extract_text_from_content(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let mut parts = Vec::new();
            for item in arr {
                let type_str = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match type_str {
                    "text" => {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            parts.push(text.to_string());
                        }
                    }
                    "thinking" => {
                        if let Some(text) = item.get("thinking").and_then(|t| t.as_str()) {
                            parts.push(format!("[thinking] {text}"));
                        }
                    }
                    "tool_use" => {
                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                        parts.push(format!("[tool: {name}]"));
                    }
                    "tool_result" => {
                        if let Some(text) = item.get("content").and_then(|c| c.as_str()) {
                            let preview = truncate_str(text, 200);
                            parts.push(format!("[result] {preview}"));
                        }
                    }
                    _ => {}
                }
            }
            parts.join("\n")
        }
        _ => String::new(),
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_line_ranges_basic() {
        let data = b"line1\nline2\nline3";
        let ranges = find_line_ranges(data);
        assert_eq!(ranges.len(), 3);
        assert_eq!(&data[ranges[0].0..ranges[0].1], b"line1");
        assert_eq!(&data[ranges[1].0..ranges[1].1], b"line2");
        assert_eq!(&data[ranges[2].0..ranges[2].1], b"line3");
    }

    #[test]
    fn test_find_line_ranges_trailing_newline() {
        let data = b"line1\nline2\n";
        let ranges = find_line_ranges(data);
        assert_eq!(ranges.len(), 2);
    }

    #[test]
    fn test_find_line_ranges_empty() {
        let ranges = find_line_ranges(b"");
        assert_eq!(ranges.len(), 0);
    }

    #[test]
    fn test_find_line_ranges_skip_blank() {
        let data = b"line1\n\nline3\n";
        let ranges = find_line_ranges(data);
        assert_eq!(ranges.len(), 2);
    }

    #[test]
    fn test_extract_text_string() {
        let val = serde_json::json!("hello world");
        assert_eq!(extract_text_from_content(&val), "hello world");
    }

    #[test]
    fn test_extract_text_array() {
        let val = serde_json::json!([
            {"type": "text", "text": "Hello"},
            {"type": "thinking", "thinking": "hmm"},
            {"type": "tool_use", "name": "Bash"}
        ]);
        let text = extract_text_from_content(&val);
        assert!(text.contains("Hello"));
        assert!(text.contains("[thinking] hmm"));
        assert!(text.contains("[tool: Bash]"));
    }

    #[test]
    fn test_search_json_value() {
        let val = serde_json::json!({"nested": {"text": "Hello World"}});
        assert!(search_json_value(&val, "hello"));
        assert!(!search_json_value(&val, "goodbye"));
    }
}
