use std::{fmt::Display, io};

use actix_web::{http::header::ContentType, HttpResponse, ResponseError};

#[derive(Debug)]
pub enum Error {
    BackendError(reqwest::Error),
    IoError(io::Error),
    InvalidConfig(toml::de::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BackendError(e) => f.write_fmt(format_args!("Backend error: {:?}", e)),
            Error::IoError(e) => f.write_fmt(format_args!("IO Error: {:?}", e)),
            Error::InvalidConfig(e) => {
                f.write_fmt(format_args!("Invalid configuration: {}", e.message()))
            }
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::BackendError(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::InvalidConfig(value)
    }
}
