use std::collections::HashMap;
use std::path::PathBuf;

use config::Config;
use libs::latex2mathml::{latex_to_mathml, DisplayStyle};
use libs::tera::{from_value, Error, Function as TeraFn, Result, Value};
use libs::toml::Value as Toml;
use utils::fs::read_file;

#[derive(Debug)]
pub struct MathTeX {
    config: Config,
}

impl MathTeX {
    #[inline]
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[derive(Debug)]
enum DataSource {
    Path(PathBuf),
    Literal(String),
}

impl DataSource {
    fn from_args(path: Option<String>, literal: Option<String>) -> Result<Self> {
        match (path, literal) {
            (Some(path), None) => Ok(Self::Path(PathBuf::from(path))),
            (None, Some(literal)) => Ok(Self::Literal(literal)),
            _ => Err(Error::msg("`mathtex` requires either a `path` or a `literal` argument.")),
        }
    }

    fn read(self) -> Result<String> {
        match self {
            Self::Path(path) => {
                read_file(&path).map_err(|e| Error::chain("Failed to read file", e))
            }
            Self::Literal(s) => Ok(s),
        }
    }
}

impl TeraFn for MathTeX {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let latex = DataSource::from_args(
            optional_arg!(String, args.get("path"), "`mathtex`: `path` must be a string."),
            optional_arg!(String, args.get("literal"), "`mathtex`: `literal` must be a string."),
        )?
        .read()?;
        let style =
            optional_arg!(String, args.get("style"), "`mathtex`: `style` must be a string.")
                .map(|s| match s.as_str() {
                    "block" | "Block" => DisplayStyle::Block,
                    _ => DisplayStyle::Inline,
                })
                .unwrap_or_else(|| get_default_style(&self.config.extra));

        match latex_to_mathml(latex.as_str(), style) {
            Ok(s) => Ok(Value::String(s)),
            Err(e) => Err(Error::chain("Failed to convert LaTeX to MathTeX", e)),
        }
    }
}

fn get_default_style(extra: &HashMap<String, Toml>) -> DisplayStyle {
    match extra.get("style") {
        Some(Toml::String(s)) => match s.as_str() {
            "block" | "Block" => DisplayStyle::Block,
            _ => DisplayStyle::Inline,
        },
        _ => DisplayStyle::Inline,
    }
}
