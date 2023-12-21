use poem_openapi::types::MaybeUndefined;
use restructed::Models;


// Should makes it impl poem's Object
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
fn mapping() {
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

    assert!(true)
}
