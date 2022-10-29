use druid::text::{
    format::{Formatter, Validation, ValidationError},
    Selection,
};
use std::num::NonZeroUsize;

pub struct NonZeroFormatter;

impl Formatter<usize> for NonZeroFormatter {
    fn format(&self, value: &usize) -> String {
        value.to_string()
    }

    fn validate_partial_input(&self, input: &str, _sel: &Selection) -> Validation {
        match input.parse::<NonZeroUsize>() {
            Ok(_) => Validation::success(),
            Err(e) => Validation::failure(e),
        }
    }

    fn value(&self, input: &str) -> Result<usize, ValidationError> {
        let num = input
            .parse::<NonZeroUsize>()
            .map_err(ValidationError::new)?;
        Ok(num.into())
    }
}
