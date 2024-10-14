use {
    num_derive::FromPrimitive,
    solana_program::{
        decode_error::DecodeError,
        msg,
        program_error::{PrintProgramError, ProgramError},
    },
    thiserror::Error,
};

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TrackerError {
    #[error("This program is absolutely deaf to your requests")]
    Deaf,
}

impl From<TrackerError> for ProgramError {
    fn from(e: TrackerError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for TrackerError {
    fn type_of() -> &'static str {
        "TrackerError"
    }
}

impl PrintProgramError for TrackerError {
    fn print<E>(&self)
    where
        E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        match self {
            TrackerError::Deaf => {
                msg!("Error: This program is absolutely deaf to your requests")
            }
        }
    }
}
