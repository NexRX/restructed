use poem_openapi::{payload::Json, types::MaybeUndefined, ApiResponse};
use restructed::Models;

//---------- Structs

// Should makes it impl poem's Object for structs
#[derive(Clone, Models)]
#[patch(UserUpdates, omit(id))]
struct User {
    id: i32,
    display_name: Option<String>,
    bio: Option<String>,
    extra: Option<String>,
    password: Option<String>,
}

impl User {
    pub fn new() -> Self {
        User {
            id: 123,
            display_name: Some("Cool Doode".to_string()),
            bio: None,
            extra: Some("Something?".to_string()),
            password: Some("Pls don't hack me".to_string()),
        }
    }
}

#[test]
fn mapping_struct() {
    let user = User::new();

    let mut update: UserUpdates = user.clone().into();
    assert_eq!(update.bio, MaybeUndefined::Undefined);
    assert_eq!(
        update.extra,
        MaybeUndefined::Value(user.extra.clone().unwrap())
    );

    // This should cause a change
    update.display_name = MaybeUndefined::Null;
    // This should *not* cause a change
    update.password = MaybeUndefined::Undefined;

    let updated_user = update.merge(user.clone());
    assert_eq!(updated_user.id, user.id);
    assert_eq!(updated_user.display_name, None);
    assert_eq!(updated_user.password, user.password);
}

//---------- Enums

// There are many derives for enums in poem_openapi.
// so you have to specify the correct derive each time sadly or it won't understand the oai attributes on the generated enum
#[derive(Debug, Default, Clone, Models, ApiResponse)]
#[view(
    ApiErrorReads,
    fields(NotFound, InternalServerError),
    default_derives = false, // poem_openapi types don't implement stuff like PartialEq, this avoids errors
    derive(ApiResponse, Clone)
)]
pub enum ApiError {
    #[oai(status = 404)]
    NotFound(Json<String>),
    #[oai(status = 409)]
    ConflictX(Json<String>),
    #[oai(status = 409)]
    ConflictY(Json<u64>),
    #[oai(status = 401)]
    Unauthorized,
    #[default]
    #[oai(status = 500)]
    InternalServerError,
}

#[test]
fn mapping_enum() {
    let reads = ApiErrorReads::NotFound(Json("No User".to_string()));

    let err: ApiError = reads.into();

    let is_match = matches!(err.clone(), ApiError::NotFound(v) if v.0 == "No User");
    assert!(is_match, "Expected NotFound, got {:?}", err);
}
