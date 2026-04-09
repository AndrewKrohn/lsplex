use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

#[derive(Debug)]
pub enum LspMsgError {
    ParsingFailed,
    IoFailed,
    Eof,
}

#[derive(Clone)]
pub enum LspMessage {
    /// id, method, full json
    Result(i32, String, Value),
    /// id, method, full json
    Request(i32, String, Value),
    /// method, full json
    Notification(String, Value),
}

impl LspMessage {
    /// Read a new lsp message from a bufreader
    pub async fn from_reader<R>(reader: &mut BufReader<R>) -> Result<Self, LspMsgError>
    where
        R: AsyncReadExt + Unpin,
    {
        let mut content_length: usize = 0;

        loop {
            let mut line = String::new();
            // `read_line` includes the trailing newline (`\n`).
            let n = reader
                .read_line(&mut line)
                .await
                .map_err(|_| LspMsgError::IoFailed)?;
            if n == 0 {
                return Err(LspMsgError::Eof);
            }

            // Empty line marks the end of the header section.
            let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
            if trimmed.is_empty() {
                break;
            }

            // mandatory `Content-Length` header.
            // todo check for other headers
            if let Some(num) = trimmed.strip_prefix("Content-Length:") {
                content_length = num
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| LspMsgError::ParsingFailed)?;
            }
        }

        if content_length == 0 {
            return Err(LspMsgError::ParsingFailed);
        }

        let mut payload = vec![0u8; content_length];
        reader
            .read_exact(&mut payload)
            .await
            .map_err(|_| LspMsgError::IoFailed)?;

        let json_data: Value =
            serde_json::from_slice(&payload).map_err(|_| LspMsgError::ParsingFailed)?;
        let method: String = if let Some(Value::String(val)) = json_data.get("method") {
            val.to_string()
        } else {
            "".to_string()
        };
        if let Some(id_val) = json_data.get("id") {
            match id_val {
                Value::Number(number) => {
                    if let Some(ival) = number.as_i64() {
                        let id: i32 = ival.try_into().map_err(|_| LspMsgError::ParsingFailed)?;
                        if json_data.get("result").is_some() {
                            return Ok(LspMessage::Result(id, method, json_data));
                        } else {
                            return Ok(LspMessage::Request(id, method, json_data));
                        }
                    }
                    return Err(LspMsgError::ParsingFailed);
                }
                _ => {
                    return Err(LspMsgError::ParsingFailed);
                }
            }
        } else {
            return Ok(LspMessage::Notification(method, json_data));
        }
    }

    /// Merge with other lsp message
    pub fn merge(&mut self, other: Self) {
        match (self, other) {
            (LspMessage::Result(_, _, my_value), LspMessage::Result(_, _, other_value)) => {
                let new_val = merge_json(my_value.clone(), other_value);
                *my_value = new_val;
            }
            (LspMessage::Notification(_, my_value), LspMessage::Notification(_, other_value)) => {
                let new_val = merge_json(my_value.clone(), other_value);
                *my_value = new_val;
            }
            _ => eprintln!("Error: attempted merge with request message"),
        }
    }

    // Format as an lsp message with header
    pub fn format_with_header(&self) -> String {
        match self {
            LspMessage::Result(_, _, value) | LspMessage::Request(_, _, value) => {
                let payload = value.to_string().into_bytes();
                let length = payload.len();
                format!(
                    "Content-Length: {}\r\n\r\n{}",
                    length,
                    String::from_utf8_lossy(&payload)
                )
            }
            LspMessage::Notification(_, value) => {
                let payload = value.to_string().into_bytes();
                let length = payload.len();
                format!(
                    "Content-Length: {}\r\n\r\n{}",
                    length,
                    String::from_utf8_lossy(&payload)
                )
            }
        }
    }
}

/// Recursively merge values.
/// Nested objects get merged recursively, arrays are merged, and if there are any differences,
/// the second Value takes precedence.
fn merge_json(a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::Object(mut map_a), Value::Object(map_b)) => {
            for (key, value_b) in map_b {
                let entry = map_a.entry(key).or_insert(Value::Null);
                *entry = merge_json(std::mem::take(entry), value_b);
            }
            Value::Object(map_a)
        }
        (Value::Array(mut arr_a), Value::Array(arr_b)) => {
            arr_a.extend(arr_b);
            Value::Array(arr_a)
        }
        (_, b) => b, // Second value wins for all other cases
    }
}
