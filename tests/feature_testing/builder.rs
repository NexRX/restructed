use restructed::Models;

#[derive(Models, Clone)]
#[patch(UserUpdatables, omit(id), default_derives = true)]
#[view(UserId, fields(id))]
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
fn can_build() {
    let user = User::new();

    let id = UserId::builder()
        .id(user.id)
        .build();
    assert_eq!(&id.id, &user.id);

    UserUpdatables::builder()
        .display_name(Some("Cooler doode".to_string()))
        .bio(None)
        .password(Some("Can't hack 'dis".to_string()))
        .build();
}