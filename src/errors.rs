use {
    num_derive::FromPrimitive,
    pinocchio::error::{ProgramError, ToStr},
    thiserror::Error,
};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PinocchioError {
    #[error("Not the signer")]
    NotSigner,
    #[error("Invalid owner account")]
    InvalidOwner,
    #[error("Invalid account data")]
    InvalidAccountData,
    #[error("Invalid address")]
    InvalidAddress,
}
//convert error to custom
impl From<PinocchioError> for ProgramError {
    fn from(error: PinocchioError) -> Self {
        ProgramError::Custom(error as u32)
    }
}

//custom --> error enum
impl TryFrom<u32> for PinocchioError {
    type Error = ProgramError;
    fn try_from(error: u32) -> Result<Self, Self::Error> {
        match error {
            0 => Ok(PinocchioError::NotSigner),
            1 => Ok(PinocchioError::InvalidOwner),
            2 => Ok(PinocchioError::InvalidAccountData),
            3 => Ok(PinocchioError::InvalidAddress),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}

impl ToStr for PinocchioError {
    fn to_str(&self) -> &'static str {
        match self {
            PinocchioError::NotSigner => "Error: Not signer of the transaction",
            PinocchioError::InvalidOwner => "Error: Invalid account owner",
            PinocchioError::InvalidAccountData => "Error: Invalid account data",
            PinocchioError::InvalidAddress => "Error: Invalid address",
        }
    }
}
