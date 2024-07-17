pub enum ValidationError {
    TooShort,
}

pub fn validate_message(message: &str) -> Result<(), ValidationError> {
    if message.len() < 1 {
        return Err(ValidationError::TooShort);
    }

    Ok(())
}
