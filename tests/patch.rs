#![allow(dead_code)]
extern crate restructed;

use restructed::Models;

#[derive(Models, Clone)]
#[patch(UserUpdatables, omit(id))]
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
