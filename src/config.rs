use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let contents = fs::read_to_string(&path).map_err(|source| ConfigError::Io {
            path: path.clone(),
            source,
        })?;
        Ok(Self {
            values: parse_values(&path, &contents)?,
        })
    }

    pub fn require(&self, key: &str) -> Result<&str, ConfigError> {
        self.values
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| ConfigError::MissingKey {
                key: key.to_owned(),
            })
    }
}

fn parse_values(path: &Path, contents: &str) -> Result<HashMap<String, String>, ConfigError> {
    let mut values = HashMap::new();
    for (line_index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| ConfigError::MalformedLine {
                path: path.to_path_buf(),
                line: line_index + 1,
            })?;
        let key = key.trim();
        if key.is_empty() {
            return Err(ConfigError::MalformedLine {
                path: path.to_path_buf(),
                line: line_index + 1,
            });
        }
        let value = value.trim().trim_matches(&['"', '\''][..]);
        values.insert(key.to_owned(), value.to_owned());
    }
    Ok(values)
}

#[derive(Debug)]
pub enum ConfigError {
    Io { path: PathBuf, source: io::Error },
    MalformedLine { path: PathBuf, line: usize },
    MissingKey { key: String },
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            Self::MalformedLine { path, line } => {
                write!(formatter, "malformed line {line} in {}", path.display())
            }
            Self::MissingKey { key } => {
                write!(formatter, "required configuration key {key} is missing")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
