use actix_web::{http::header::ContentType, HttpResponse, ResponseError};
use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LBError {
    #[error("backend error")]
    BackendError(#[from] reqwest::Error),

    #[error("Configuration file not found at \"{:?}\"", .config_file_path.as_path())]
    MissingConfigurationFile {
        config_file_path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Invalid configuration file")]
    InvalidConfig(#[from] toml::de::Error),

    #[error("Generic I/O error")]
    IoError(#[from] io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ResponseError for LBError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
}
