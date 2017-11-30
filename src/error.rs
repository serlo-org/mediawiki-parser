use ast;
use std::fmt;
use std::error;
use colored::*;
use util;
use grammar;

/// The number of lines to display as error context.
const ERROR_CONTEXT_LINES: usize = 5;



/// The parser error with source code context.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="lowercase", deny_unknown_fields)]
pub struct ParseError {
    pub position: ast::Position,
    pub expected: Vec<String>,
    pub context: Vec<String>,
    pub context_start: usize,
    pub context_end: usize,
}

/// Error structure for syntax tree transformations.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="lowercase", deny_unknown_fields)]
pub struct TransformationError {
    pub cause: String,
    pub position: ast::Span,
    pub transformation_name: String,
    pub tree: ast::Element,
}

impl ParseError {
    pub fn from(err: &grammar::ParseError, input: &str) -> Self {

        let source_lines = util::get_source_lines(&input);
        let line_count = source_lines.len();

        let line = if err.line <= line_count {
            err.line
        } else {
            source_lines.len()
        } - 1;

        let start = if line < ERROR_CONTEXT_LINES {
            0
        } else {
            line - ERROR_CONTEXT_LINES
        };

        let end = if line + ERROR_CONTEXT_LINES >= line_count {
            line_count - 1
        } else {
            line + ERROR_CONTEXT_LINES
        };

        let mut token_str = vec![];
        for token in &err.expected {
            token_str.push(String::from(*token));
        }


        let mut context = vec![];
        for sloc in source_lines[start..end + 1].iter() {
            context.push(String::from(sloc.content));
        }

        ParseError {
            position: ast::Position::new(err.offset, &source_lines),
            context: context,
            expected:  token_str,
            context_start: start,
            context_end: end,
        }
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        "Could not continue to parse, because no rules could be matched."
    }
}

impl fmt::Display for ParseError {

    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let error_message = format!("ERROR in line {} at column {}: Could not continue to parse, expected one of: ",
            self.position.line, self.position.col).red().bold();

        let mut token_str = vec![];
        for token in &self.expected {
            if util::is_whitespace(token) {
                token_str.push(format!("{:?}", token));
            } else {
                token_str.push(format!("{}", token));
            }
        }

        write!(f, "{}", error_message)?;
        write!(f, "{}\n", token_str.join(", ").blue().bold())?;

        for (i, content) in self.context.iter().enumerate() {

            let lineno = format!("{} |", self.context_start + i + 1);
            let lineno_col;

            let formatted_content;
            // the erroneous line
            if self.context_start + i + 1 == self.position.line {
                formatted_content = content.red();
                lineno_col = lineno.red().bold();
            } else {
                formatted_content = util::shorten_str(content).normal();
                lineno_col = lineno.blue().bold()
            }

            writeln!(f, "{} {}", lineno_col, formatted_content)?;
        }

        Ok(())
    }
}

impl error::Error for TransformationError {
    fn description(&self) -> &str {
        &self.cause
    }
}

impl fmt::Display for TransformationError {

    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let message = format!("ERROR applying transformation \"{}\" to Elemtn at {}:{} to {}:{}: {}",
            self.transformation_name, self.position.start.line, self.position.start.col,
            self.position.end.line, self.position.end.col, self.cause
        );
        writeln!(f, "{}", message.red().bold())
    }
}