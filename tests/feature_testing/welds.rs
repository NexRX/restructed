use restructed::Models;
use welds::state::DbState;

#[derive(Models, Clone, PartialEq, Eq, Debug)]
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

    let patch = UserUpdatables::builder()
        .display_name(Some("Cooler doode".to_string()))
        .bio(None)
        .password(Some("Can't hack 'dis".to_string()))
        .build();

    let mut state = DbState::new_uncreated(user.clone());
    patch.merge_weld_mut(&mut state);

    
    assert_ne!(*state, user);
    assert_eq!(*state.display_name, "Cooler doode".to_string());
    assert_eq!(*state.bio, "I'm a cool doode, what can I say?".to_string());
    assert_eq!(*state.password, "Can't hack 'dis".to_string());
}