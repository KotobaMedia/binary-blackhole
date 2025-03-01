use aws_sdk_dynamodb::{error::SdkError, operation};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DataError>;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("serde_dynamo Error: {0}")]
    SerdeDynamoError(#[from] serde_dynamo::Error),

    #[error("DynamoDB PutItem Error: {0}")]
    DynamoPutItemError(#[from] SdkError<operation::put_item::PutItemError>),
    #[error("DynamoDB GetItem Error: {0}")]
    DynamoGetItemError(#[from] SdkError<operation::get_item::GetItemError>),
    #[error("DynamoDB Query Error: {0}")]
    DynamoQueryError(#[from] SdkError<operation::query::QueryError>),

    #[error("Document not found")]
    DocumentNotFound,
}
