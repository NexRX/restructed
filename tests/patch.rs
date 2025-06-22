#![allow(dead_code)]
extern crate restructed;

use poem_openapi::types::MaybeUndefined;
use restructed::Models;
use serde_json::{json, to_value};

#[derive(Models, Clone)]
#[patch(UserUpdatables, omit(id))]
struct User {
    id: i32,
    display_name: String,
    bio: Option<String>,
    password: String,
}

impl User {
    pub fn new() -> Self {
        User {
            id: 123,
            display_name: "Cool Doode".to_string(),
            bio: Some("I'm a cool doode, what can I say?".to_string()),
            password: "Pls don't hack me".to_string(),
        }
    }
}

#[test]
fn omitted_only() {
    let user = User::new();

    let updates = UserUpdatables {
        display_name: Some("Cooler doode".to_string()),
        bio: None,
        password: Some("Can't hack 'dis".to_string()),
    };

    let updated_user = updates.merge(user.clone());

    assert_ne!(user.display_name, updated_user.display_name);
    assert_eq!(user.bio, updated_user.bio);
    assert_ne!(user.password, updated_user.password);
}

//------------------ Structs - MaybeUndefined


#[derive(Models, Clone)]
#[patch(UserMaybes, omit(id), option = MaybeUndefined)]
struct UserAlt {
    id: i32,
    display_name: String,
    bio: Option<String>,
    password: String,
}

impl UserAlt {
    pub fn new() -> Self {
        UserAlt {
            id: 123,
            display_name: "Cool Doode".to_string(),
            bio: Some("I'm a cool doode, what can I say?".to_string()),
            password: "Pls don't hack me".to_string(),
        }
    }
}

//------------------ Structs -- defaults

#[derive(Models)]
#[model(defaults(fields(display_name, bio), attributes_with = "none"))]
#[patch(UserProfileDefaults)]
struct UserDefaults{
    id: i32,
    display_name: String,
    bio: String,
    password: String,
}

//------------------ Structs -- base
#[derive(Models)]
#[model(base(fields(display_name, bio), attributes_with = "none"))]
#[patch(UserProfileBase)]
struct UserBase {
    id: i32,
    display_name: String,
    bio: String,
    password: String,
}

//------------------ Structs -- base & defaults mix
#[derive(Models)]
#[model(base(fields(bio, display_name), attributes_with = "none"), defaults(omit(display_name)))]
#[patch(UserProfileMix)]
struct UserMix {
    id: i32,
    display_name: String,
    bio: String,
    password: String,
}


#[test]
fn alt_omitted_only() {
    let user = UserAlt::new();

    let maybes = UserMaybes {
        display_name: Some("Cooler doode".to_string()),
        bio: MaybeUndefined::Null,
        password: Some("Can't hack 'dis".to_string()),
    };

    let updated_user = maybes.merge(user.clone());

    assert_ne!(user.display_name, updated_user.display_name);
    assert_ne!(user.bio, updated_user.bio);
    assert_ne!(user.password, updated_user.password);
}


// ------------------------------

#[derive(restructed::Models)]
#[patch(UserPatch, attributes_with = "all". skip_serializing_double_option = true)]
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
struct UserDontSerialName {
    name: Option<String>,
    email: String,
}

#[test]
fn name_patch_serialization() {
    let name_some = UserPatch {
        name: Some(Some("c00l dud3".to_string())),
        email: Some("www.com".to_string()),
    };

    let name_none = UserPatch {
        name: None,
        email: Some("www.com".to_string()),
    };

    // When name is Some(Some(_)), it should serialize
    let value_some = to_value(&name_some).unwrap();
    assert_eq!(value_some.get("name"), Some(&json!("c00l dud3")));

    // When name is None, it should not serialize the field
    let value_none = to_value(&name_none).unwrap();
    assert!(value_none.get("name").is_none());
}