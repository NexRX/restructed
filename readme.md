# restructed

<!-- Credit to Poem crate for this readme.md section! I love that crate! -->
<div align="center">
  <a href="https://github.com/NexRX/restructed">
    <img src="https://img.shields.io/badge/GitHub-100000?style=for-the-badge&logo=github&logoColor=white"
      alt="GitHub Repo" />
  </a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/restructed">
    <img src="https://img.shields.io/crates/v/restructed.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/restructed">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <a href="https://github.com/rust-secure-code/safety-dance/">
    <img src="https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square"
      alt="Unsafe Rust Forbidden" />
  </a>
</div>

<br/>

A quick and easy way to create derivative models of your existing types without repeating yourself. Reduce boilerplate and automatically generate related structs with custom field subsets and transformations.

## Features

- **Reduce boilerplate**: Generate multiple related structs from a single definition
- **Flexible field selection**: Include or exclude specific fields with `fields()` and `omit()`
- **Automatic trait generation**: `From<T>` implementations between original and generated structs
- **Derive support**: Apply derives to generated structs
- **Multiple model types**: Views, patches, and custom transformations

New features and roadmap are available [here](https://github.com/NexRX/restructed/issues/1) on GitHub.

## Installation

Add `restructed` to your `Cargo.toml`:

```toml
[dependencies]
restructed = "0.2"
```

Or run this command in your project directory:

```sh
cargo add restructed
```

## Quick Start

Add the derive macro to your struct:

```rust
use restructed::Models;

#[derive(restructed::Models)]
struct User {
    id: i32,
    username: String,
    email: String,
    password: String,
}
```

Then add attributes for each model you want to generate:

```rust
#[derive(restructed::Models)]
#[view(UserProfile, omit(password))]           // Subset without sensitive fields
#[patch(UserUpdate, fields(username, email))]  // Optional fields for updates
struct User {
    id: i32,
    username: String,
    email: String,
    password: String,
}
```

This generates:

- `UserProfile` struct with `id`, `username`, and `email` fields
- `UserUpdate` struct with `Option<String>` fields for `username` and `email`
- Automatic `From` implementations for conversions

## Model Types

### `#[view]` - Field Subsets

Creates a struct containing a subset of the original fields. Perfect for API responses, database views, or public representations.

**Arguments:**

| Name              | Description                  | Required    | Type       | Example                   |
| ----------------- | ---------------------------- | ----------- | ---------- | ------------------------- |
| `name`            | Name of the generated struct | Yes (first) | Identifier | `UserProfile`             |
| `fields`          | Fields to include            | No          | List       | `fields(id, username)`    |
| `omit`            | Fields to exclude            | No          | List       | `omit(password, secret)`  |
| `derive`          | Traits to derive             | No          | List       | `derive(Debug, Clone)`    |
| `preset`          | Behavior preset to apply     | No          | String     | `preset = "read"`         |
| `attributes_with` | Attributes to inherit        | No          | String     | `attributes_with = "all"` |

**Note:** Use either `fields` OR `omit`, not both.

**Example:**

```rust
#[derive(Clone, restructed::Models)]
#[view(UserProfile, omit(id, password))]
struct User {
    id: i32,           // Not in UserProfile
    username: String,  // In UserProfile
    email: String,     // In UserProfile
    bio: String,       // In UserProfile
    password: String,  // Not in UserProfile
}

// Usage
let user = User {
  id: 1,
  username: "alice".to_string(),
  email: "alice@example.com".to_string(),
  bio: "Rustacean".to_string(),
  password: "super_secret".to_string(),
};
let profile: UserProfile = user.into();
```

### `#[patch]` - Optional Field Wrappers

Creates a struct where each field is wrapped in `Option<T>`. Ideal for partial updates, PATCH endpoints, or optional modifications.

**Arguments:**

| Name                             | Description                                     | Required    | Type       | Example                                 |
| -------------------------------- | ----------------------------------------------- | ----------- | ---------- | --------------------------------------- |
| `name`                           | Name of the generated struct                    | Yes (first) | Identifier | `UserUpdate`                            |
| `fields`                         | Fields to include                               | No          | List       | `fields(username, email)`               |
| `omit`                           | Fields to exclude                               | No          | List       | `omit(id, created_at)`                  |
| `derive`                         | Traits to derive                                | No          | List       | `derive(Debug, Serialize)`              |
| `preset`                         | Behavior preset to apply                        | No          | String     | `preset = "write"`                      |
| `attributes_with`                | Attributes to inherit                           | No          | String     | `attributes_with = "oai"`               |
| `option`                         | Alternative to `Option<T>`                      | No          | Type       | `option = MaybeUndefined`               |
| `skip_serializing_double_option` | Skip serializing `None` for `Option<Option<T>>` | No          | Boolean    | `skip_serializing_double_option = true` |

**Example:**

```rust
#[derive(Clone, restructed::Models)]
#[patch(UserUpdate, omit(id))]
struct User {
    id: i32,                    // Not in UserUpdate
    username: String,           // Option<String> in UserUpdate
    email: String,              // Option<String> in UserUpdate
    bio: Option<String>,        // Option<Option<String>> in UserUpdate
}

// Usage
let update = UserUpdate {
    username: Some("new_username".to_string()),
    email: None,  // Don't update email
    bio: Some(Some("New bio".to_string())),  // Set bio
};
```

### `#[model]` - Base Configuration

Defines default arguments applied to all generated models. This attribute doesn't generate structs itself but configures other model generators.

**Sub-attributes:**

#### `base` - Non-overridable Defaults

Arguments that are always applied and cannot be overridden by individual models.

```rust
#[model(base(derive(Debug, Clone)))]  // All models MUST derive Debug and Clone
```

#### `defaults` - Overridable Defaults

Arguments applied only when not specified by individual models.

```rust
#[model(defaults(derive(Serialize), preset = "read"))]  // Applied unless overridden
```

**Example:**

```rust
#[derive(restructed::Models)]
#[model(
    base(derive(Debug)),                     // All models derive Debug
    defaults(derive(Clone), preset = "read") // Default unless overridden
)]
#[view(UserView)]                           // Inherits Debug + Clone + preset="read"
#[patch(UserPatch, preset = "write")]       // Inherits Debug + Clone, overrides preset
struct User {
    id: i32,
    username: String,
    password: String,
}
```

## Advanced Features

### Presets

Presets apply common configurations automatically:

- **`"none"`** (default): No special behavior
- **`"write"`** _(requires 'openapi' feature)_: For writable fields
  - Removes `#[oai(read_only)]` fields
  - Uses `MaybeUndefined` for patch option type
- **`"read"`** _(requires 'openapi' feature)_: For readable fields
  - Removes `#[oai(write_only)]` fields
  - Uses `MaybeUndefined` for patch option type

### Attribute Inheritance

Control which attributes are copied to generated structs:

- **`"none"`** (default): No attributes copied
- **`"oai"`** _(requires 'openapi' feature)_: Copy OpenAPI attributes
- **`"deriveless"`**: Copy all attributes except derives
- **`"all"`**: Copy all attributes, even dervies but they'll need to be on their own line (See below example) i.e.

    ```rust
    #[derive(restructed::Models)]
    #[model(defaults(attributes_with = "all"))]
    #[derive(Clone)]
    ```

### Complete Example

```rust
#[derive(restructed::Models, Clone)]
#[view(UserProfile, omit(password, internal_id))]
#[view(UserSummary, fields(id, username))]
#[patch(UserUpdate, omit(id, internal_id, created_at), preset = "write")]
struct User {
    id: i32,
    internal_id: String,
    username: String,
    email: String,
    password: String,
    created_at: String,
}

fn example_usage() {
    let user = User {
        id: 1,
        internal_id: "internal_123".to_string(),
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        password: "secret".to_string(),
        created_at: "2024-01-01".to_string(),
    };

    // Convert to different views
    let profile: UserProfile = user.clone().into();
    let summary: UserSummary = user.clone().into();

    // Create update struct
    let update = UserUpdate {
        username: Some("new_alice".to_string()),
        email: None,  // Don't update
        password: Some("new_secret".to_string()),
    };
}
```

## Feature Flags

### `openapi` - Poem OpenAPI Integration

Enables integration with the [poem-openapi](https://github.com/poem-web/poem/tree/master/poem-openapi) crate:

- Use `MaybeUndefined<T>` instead of `Option<T>` in patch models
- Copy `#[oai(...)]` attributes to generated structs
- Respect `read_only`/`write_only` attributes in presets

```rust
use restructed::Models;

#[derive(poem_openapi::Object, Models)]
#[oai(skip_serializing_if_is_none, rename_all = "camelCase")]
#[model(
    base(derive(poem_openapi::Object, Debug)),
    defaults(preset = "read")
)]
#[patch(UserUpdate, preset = "write")]
#[view(UserProfile)]
#[view(UserNames, fields(username, name, surname))]
pub struct User {
    #[oai(read_only)]
    pub id: u32,

    #[oai(validator(min_length = 3, max_length = 16, pattern = r"^[a-zA-Z0-9_]*$"))]
    pub username: String,

    #[oai(validator(min_length = 5, max_length = 1024), write_only)]
    pub password: String,

    #[oai(validator(min_length = 2, max_length = 16, pattern = r"^[a-zA-Z\s]*$"))]
    pub name: Option<String>,

    #[oai(validator(min_length = 2, max_length = 16, pattern = r"^[a-zA-Z\s]*$"))]
    pub surname: Option<String>,

    #[oai(read_only)]
    pub joined: u64,
}
```

## Limitations

- **Generic types**: Currently doesn't support generic structs or enums (e.g., `Struct<T>`)
- **Enum support**: Only works with structs, not enums

Contributions for these features are welcome!

## License

See the project repository for license information.
