use std::collections::HashMap;
use std::io;

#[derive(Debug, Default)]
pub struct Headers {
    map: HashMap<String, String>,
}

impl Headers {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.map.get(&key.to_ascii_lowercase())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.map.insert(key.to_ascii_lowercase(), value.to_string());
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.map.iter()
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.map.remove(&key.to_ascii_lowercase())
    }

    pub fn parse(&mut self, data: &[u8]) -> io::Result<(usize, bool)> {
        let Some(pos) = find_crlf(data) else {
            return Ok((0, false));
        };

        if pos == 0 {
            return Ok((2, true));
        }

        let line = std::str::from_utf8(&data[..pos])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8"))?;

        let Some(colon_index) = line.find(':') else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "missing colon"));
        };

        let key = &line[..colon_index];
        let value = &line[colon_index + 1..];

        if key.trim() != key || key.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid header spacing",
            ));
        }

        if !is_valid_header_key(key) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid header name",
            ));
        }

        let key = key.to_ascii_lowercase();
        let value = value.trim();

        if let Some(existing_value) = self.map.get_mut(&key) {
            existing_value.push_str(", ");
            existing_value.push_str(value);
        } else {
            self.map.insert(key, value.to_string());
        }

        Ok((pos + 2, false))
    }
}

fn find_crlf(data: &[u8]) -> Option<usize> {
    data.windows(2).position(|w| w == b"\r\n")
}

fn is_valid_header_key(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }

    key.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(
                c,
                '!' | '#'
                    | '$'
                    | '%'
                    | '&'
                    | '\''
                    | '*'
                    | '+'
                    | '-'
                    | '.'
                    | '^'
                    | '_'
                    | '`'
                    | '|'
                    | '~'
            )
    })
}

#[cfg(test)]
mod tests;
