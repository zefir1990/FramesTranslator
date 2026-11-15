use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    base_directory: PathBuf,
    values: HashMap<String, String>,
}

impl Config {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let contents = fs::read_to_string(&path).map_err(|source| ConfigError::Io {
            path: path.clone(),
            source,
        })?;
        let base_directory = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .to_path_buf();
        let values = parse_values(&path, &contents)?;
        Ok(Self {
            base_directory,
            values,
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

    pub fn require_u32(&self, key: &str) -> Result<u32, ConfigError> {
        let value = self.require(key)?;
        value
            .parse()
            .map_err(|_| invalid_value(key, value, "an unsigned 32-bit integer"))
    }

    pub fn require_usize(&self, key: &str) -> Result<usize, ConfigError> {
        let value = self.require(key)?;
        value
            .parse()
            .map_err(|_| invalid_value(key, value, "an unsigned integer"))
    }

    pub fn require_color(&self, key: &str) -> Result<[u8; 4], ConfigError> {
        let value = self.require(key)?;
        let components: Vec<&str> = value.split(',').map(str::trim).collect();
        let [red, green, blue, alpha] = components.as_slice() else {
            return Err(invalid_value(key, value, "four comma-separated bytes"));
        };
        Ok([
            parse_color_component(key, value, red)?,
            parse_color_component(key, value, green)?,
            parse_color_component(key, value, blue)?,
            parse_color_component(key, value, alpha)?,
        ])
    }

    pub fn resolve_path(&self, key: &str) -> Result<PathBuf, ConfigError> {
        let value = self.require(key)?;
        Ok(self.base_directory.join(value))
    }
}

fn parse_values(path: &Path, contents: &str) -> Result<HashMap<String, String>, ConfigError> {
    let mut values = HashMap::new();
    for (line_index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line_number = line_index + 1;
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| ConfigError::MalformedLine {
                path: path.to_path_buf(),
                line: line_number,
            })?;
        let key = key.trim();
        if key.is_empty() {
            return Err(ConfigError::MalformedLine {
                path: path.to_path_buf(),
                line: line_number,
            });
        }
        let value = value.trim().trim_matches(&['"', '\''][..]);
        values.insert(key.to_owned(), value.to_owned());
    }
    Ok(values)
}

fn parse_color_component(key: &str, value: &str, component: &str) -> Result<u8, ConfigError> {
    component
        .parse()
        .map_err(|_| invalid_value(key, value, "four comma-separated bytes"))
}

fn invalid_value(key: &str, value: &str, expected: &'static str) -> ConfigError {
    ConfigError::InvalidValue {
        key: key.to_owned(),
        value: value.to_owned(),
        expected,
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    MalformedLine {
        path: PathBuf,
        line: usize,
    },
    MalformedFileEntry {
        path: PathBuf,
        line: usize,
    },
    MissingKey {
        key: String,
    },
    InvalidValue {
        key: String,
        value: String,
        expected: &'static str,
    },
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
            Self::MalformedFileEntry { path, line } => {
                write!(
                    formatter,
                    "malformed entry at line {line} in {}",
                    path.display()
                )
            }
            Self::MissingKey { key } => {
                write!(formatter, "required configuration key {key} is missing")
            }
            Self::InvalidValue {
                key,
                value,
                expected,
            } => write!(
                formatter,
                "configuration key {key} has value {value:?}, expected {expected}"
            ),
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
