use axum::http::uri::InvalidUri;
use axum::http::method::InvalidMethod;
use axum::http::Error as HttpError;
use axum::http::header::InvalidHeaderName;
use axum::http::header::InvalidHeaderValue;
use axum::http::header::ToStrError;
use axum::{
    Error as AxumError,
};
use worker::{
    Error as WorkerError,
};

#[derive(Debug)]
// #[non_exhaustive]
pub enum Error {
    WorkerError(WorkerError),
    AxumError(AxumError),
    InvalidUri(InvalidUri),
    InvalidMethod(InvalidMethod),
    HttpError(HttpError),
    InvalidHeaderValue(InvalidHeaderValue),
    InvalidHeaderName(InvalidHeaderName),
    ToStrError(ToStrError),
}

impl From<AxumError> for Error {
    fn from(err: AxumError) -> Error {
        Error::AxumError(err)
    }
}

impl From<ToStrError> for Error {
    fn from(err: ToStrError) -> Error {
        Error::ToStrError(err)
    }
}

impl From<HttpError> for Error {
    fn from(err: HttpError) -> Error {
        Error::HttpError(err)
    }
}

impl From<InvalidMethod> for Error {
    fn from(err: InvalidMethod) -> Error {
        Error::InvalidMethod(err)
    }
}

impl From<InvalidUri> for Error {
    fn from(err: InvalidUri) -> Error {
        Error::InvalidUri(err)
    }
}

impl From<WorkerError> for Error {
    fn from(err: WorkerError) -> Error {
        Error::WorkerError(err)
    }
}

impl From<InvalidHeaderName> for Error {
    fn from(err: InvalidHeaderName) -> Error {
        Error::InvalidHeaderName(err)
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(err: InvalidHeaderValue) -> Error {
        Error::InvalidHeaderValue(err)
    }
}