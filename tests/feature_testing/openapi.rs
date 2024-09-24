#![allow(dead_code)]

use poem_openapi::{payload::Json, types::MaybeUndefined, ApiResponse, Object};
use restructed::Models;

//---------- Structs

// Should makes it impl poem's Object for structs
#[derive(Clone, Object, Models)]
#[oai(skip_serializing_if_is_none, rename_all = "camelCase")]
#[patch(UserUpdates, omit(id), attributes_with = "oai", option = MaybeUndefined, derive(Object))]
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

//---------- Structs - Preset Read/Write

// Should makes it impl poem's Object for structs
#[derive(Clone, Debug, Object, Models, PartialEq, Eq)]
#[oai(skip_serializing_if_is_none, rename_all = "camelCase")]
#[patch(UserUpdatable, preset = "write", derive(Object))]
#[patch(UserViewables, preset = "read", derive(Object))]
#[view(UserUpdated, preset = "write", derive(Object))]
#[view(UserRecord, preset = "read", derive(Object))]
struct UserPre {
    #[oai(read_only)]
    id: i32,
    display_name: Option<String>,
    bio: String,
    extra: Option<String>,
    #[oai(write_only)]
    password: Option<String>,
}

//---------- Enums

// There are many derives for enums in poem_openapi.
// so you have to specify the correct derive each time sadly or it won't understand the oai attributes on the generated enum
#[derive(Debug, Default, Clone, Models, ApiResponse)]
#[view(
    ApiErrorReads,
    fields(NotFound, InternalServerError),
    attributes_with = "oai",
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


// ---------- Issues

// https://github.com/NexRX/restructed/issues/3 - #[oai(example)] causes compilation errors with generated structs
#[derive(poem_openapi::Object, Models)]
#[oai(example, skip_serializing_if_is_none, rename_all = "camelCase")] // <-- this here
#[model(base(derive(poem_openapi::Object, Debug)), defaults(preset = "read"))]
#[patch(Issue3UserUpdate, preset = "write")]
pub struct Issue3User {
    #[oai(read_only)]
    pub id: u32,

    #[oai(validator(min_length = 3, max_length = 16, pattern = r"^[a-zA-Z0-9_]*$"))] // oai attributes carry over with `preset = write/write` or attributes_with="oai"
    pub username: String,

    #[oai(validator(min_length = 5, max_length = 1024), write_only)]
    pub password: String,
}

impl poem_openapi::types::Example for Issue3User {
    fn example() -> Self {
        unimplemented!()
    }
}