#![allow(dead_code)]

use restructed::Models;

#[derive(Models)]
#[view(UserProfile, fields(display_name, bio))]
struct User {
    id: i32,
    display_name: String,
    bio: String,
    password: String,
}

impl User {
    pub fn new() -> Self {
        User {
            id: 123,
            display_name: "Cool Doode".to_string(),
            bio: "I'm a cool doode, what can I say?".to_string(),
            password: "Pls don't hack me".to_string(),
        }
    }
}

#[test]
fn from_original() {
    let user = User::new();

    let profile: UserProfile = user.into();

    assert_eq!(profile.display_name, profile.display_name);
    assert_eq!(profile.bio, profile.bio);
}

#[test]
fn only_fields() {
    let user = User::new();

    let profile = UserProfile{
        display_name: user.display_name,
        bio: user.bio,
    };

    assert_eq!(profile.display_name, profile.display_name);
    assert_eq!(profile.bio, profile.bio);
}
