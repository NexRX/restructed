use restructed::Models;

#[derive(Models)]
#[clone(UserClone, derive(Clone))]
struct User {
    /// This should be omitted
    id: i32,
    /// This shouldn't be omitted
    display_name: String,
    bio: String,
    password: String,
}
