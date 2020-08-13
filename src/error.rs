use std::{fmt, string};

// 'a: log format, 'b: input
#[derive(Debug)]
pub enum ParseErr {
    MalformedRequestField,
    WrongSequence { expected: String, actual: String },
    FieldMismatch { expected: usize, actual: usize },
    NoField { field: String },
}

impl std::error::Error for ParseErr {}
impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tokenize error: {}",
            match self {
                ParseErr::MalformedRequestField => String::from("malformed request field"),
                ParseErr::WrongSequence { expected, actual } =>
                    format!("wrong sequence, expected: {}, actual: {}", expected, actual),
                ParseErr::FieldMismatch { expected, actual } => format!(
                    "field mismatch, expected {} fields, actual: {}",
                    expected, actual
                ),
                ParseErr::NoField { field } => format!("no such field {}", field),
            }
        )
    }
}
