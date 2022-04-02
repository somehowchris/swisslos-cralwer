use chrono::ParseError as ChronoParseError;
use reqwest::Error as ReqwestError;

#[derive(Debug)]
pub enum Errors {
    ReqwestClientError(ReqwestError),
    ParserError,
    SuppliedDateHasNoDraw,
    UnexpectedParsingError(String, String),
    DateParsingError(ChronoParseError),
}

impl From<ReqwestError> for Errors {
    fn from(e: ReqwestError) -> Self {
        Self::ReqwestClientError(e)
    }
}

impl From<ChronoParseError> for Errors {
    fn from(e: ChronoParseError) -> Self {
        Self::DateParsingError(e)
    }
}