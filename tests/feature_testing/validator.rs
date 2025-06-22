use restructed::Models;

// A trait that the Validate derive will impl
use validator::{Validate, ValidationError};

#[derive(Validate, Models, Clone)]
#[view(UserNameOnly, attributes_with = "all", derive(Validate))]
struct User {
    #[validate(length(min = 1), custom(function = "validate_unique_username"))]
    name: String,
    #[validate(range(min = 18, max = 20))]
    age: u32,
}

fn validate_unique_username(name: &str) -> Result<(), ValidationError> {
    if name == "xXxShad0wxXx" {
        // the value of the username will automatically be added later
        return Err(ValidationError::new("terrible_username"));
    }

    Ok(())
}

#[test]
fn both_invalidate() {
    let user: User = User {
        name: "xXxShad0wxXx".to_string(),
        age: 19
    };

    let view: UserNameOnly = user.clone().into();

    user.validate();
    view.validate();
}