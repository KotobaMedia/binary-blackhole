use aws_sdk_dynamodb::{error::SdkError, operation};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DataError>;

#[derive(Error, Debug)]
pub enum DataError {
    #[error(transparent)]
    SerdeDynamoError(#[from] serde_dynamo::Error),

    #[error(transparent)]
    DynamoPutItemError(#[from] SdkError<operation::put_item::PutItemError>),
    #[error(transparent)]
    DynamoGetItemError(#[from] SdkError<operation::get_item::GetItemError>),
    #[error(transparent)]
    DynamoQueryError(#[from] SdkError<operation::query::QueryError>),
}
