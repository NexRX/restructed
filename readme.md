# `restructed`

<!-- Credit to poem crate for this readme.md section! I love that crate! -->
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
      alt="Unsafe Rust forbidden" />
  </a>
</div>
<br/>

A quick and easy way to create derivative models of your existing types without repeating yourself all the damn time. <br/>

- Reduce number of structs you need to write
- Allows deriving on generated structs
- Allows generating multiple structs from one derive
- Automatically generates `From<T>` traits for original <> generated structs

New features planned are available [here](https://github.com/NexRX/restructed/issues/1) on github. See below for examples of current usage & features.

# Usage

Add `restructed` to your projects `Cargo.toml`:

```toml
restructed = "0.1"
```

alternatively run this in the project directory

```sh
cargo add restructed
```

Add the import and derive it on the target struct

```rust
#[derive(restructed::Models)]
struct User {
    id: i32,
    username: String
}

```

And then add attributes for each model you want to create.

```rust
#[derive(restructed::Models)]
#[view(UserId, fields(id))]        // <-- Simple subset of the deriving structs field
#[patch(UserUpdatables, omit(id))] // <-- Wraps all fields with a Option type inside a new struct
struct User {
    id: i32,
    username: String,
}
```

Continue reading for the available models and their breakdown and arguments in general.


# Models
These are arguments that can be applied there respective Attributes (i.e. `#[attr(args...)])`).

## `#[view]`
A model with generates a subset of the original/parent deriving `Model`. Useful for creating things like RESTful API or Database views.

| Argument Name   | description                                        | Required?  | Type/Enum               | Example                           |
|-----------------|----------------------------------------------------|------------|-------------------------|-----------------------------------|
| name            | Name of the struct the generate                    | True+First | Identifier              | `MyStruct`                        |
| **fields** or   | Field names in the original structure to include   | False      | List(Ident)             | `fields(field1, field2, ...)`     |  
| **omit**        | Field names in the original structure to exclude   | False      | List(Ident)             | `omit(field1, field2, ...)`       |
| derive          | Things to derive on the newly generated struct     | False      | List(Path)              | `derive(Debug, thiserror::Error)` |
| preset          | Behaviours and/or defaults to apply                | False      | none/write/read         | `preset = "all"`                  |
| attributes_with | Attributes to inherit at both struct & field level | False      | none/oai/deriveless/all | `attributes_with = "all"`         |

```rust
#[derive(Clone, restructed::Models)]
#[view(UserProfile, omit(id, password))]
struct User {
    id: i32, // Not in `UserProfile`
    display_name: String,
    bio: String,
    extra: Option<String>,
    password: String, // Not in `UserProfile`
}
```

## `#[patch]`
A model which creates subsets of your data except each field's type is wrapped in a `Option<t>` or a alternative type of Option implementation if specified. Useful for creating RESTful API patch method types or Database Table Patches where you only want to update fields if they were explictly given (even to delete).

| Argument Name   | description                                        | Required?  | Type/Enum               | Example                           |
|-----------------|----------------------------------------------------|------------|-------------------------|-----------------------------------|
| name            | Name of the struct the generate                    | True+First | Identifier              | `MyStruct`                        |
| **fields** or   | Field names in the original structure to include   | False      | List(Ident)             | `fields(field1, field2, ...)`     |  
| **omit**        | Field names in the original structure to exclude   | False      | List(Ident)             | `omit(field1, field2, ...)`       |
| derive          | Things to derive on the newly generated struct     | False      | List(Path)              | `derive(Debug, thiserror::Error)` |
| preset          | Behaviours and/or defaults to apply                | False      | none/write/read         | `preset = "all"`                  |
| attributes_with | Attributes to inherit at both struct & field level | False      | none/oai/deriveless/all | `attributes_with = "all"`         |
| option          | A alternative to `Option<T>` to wrap fields with   | False      | Option/MaybeUndefined   | `option = MaybeUndefined`         |

```rust
#[derive(Clone, restructed::Models)]
#[patch(UserUpdate, fields(display_name, bio, extra, password))]
struct User {
    id: i32, // Not in `UserUpdate`
    display_name: String, // Option<String> in `UserUpdate`
    bio: String, // Option<String> in `UserUpdate`
    extra: Option<String>, // Option<Option<String>> in `UserUpdate` (If this isn't desired, see *option* arg and the *openapi* crate feature)
    password: String, // Not in `UserProfile`
}
```

## `#[model]`
Not a model, used to define a *base* or *default* set of arguments to be apply to all models. Acts as a interface for taking arguments to apply more broad and *doesn't* generate any models itself.

There are two arguments possible
### base
A *list* of non-overridable arguments that are applied to all generated arguments that you cab build on-top off. It doesn't prevent you from using the in individual models later but it also won't allow you to undo the effect individually.

e.g. `#[model(base(...)]`

| Argument Name   | description                                        | Required?  | Type/Enum               | Example                           |
|-----------------|----------------------------------------------------|------------|-------------------------|-----------------------------------|
| **fields** or   | Field names in the original structure to include   | False      | List(Ident)             | `fields(field1, field2, ...)`     |  
| **omit**        | Field names in the original structure to exclude   | False      | List(Ident)             | `omit(field1, field2, ...)`       |
| derive          | Things to derive on the newly generated struct     | False      | List(Path)              | `derive(Debug, thiserror::Error)` |

### defaults
Arguments given in this list are applied to all models where the argument isn't given. Meaning, if writen `#[model(defaults(fields(a, b)))]` and then later `#[view(omit(b))]` is written, the `fields(a, b)` earlier will not be applied because the two args are mutally exclusive unlike with *base* arguments.

e.g. `#[model(defaults(...))]`

| Argument Name   | description                                        | Required?  | Type/Enum               | Example                           |
|-----------------|----------------------------------------------------|------------|-------------------------|-----------------------------------|
| **fields** or   | Field names in the original structure to include   | False      | List(Ident)             | `fields(field1, field2, ...)`     |  
| **omit**        | Field names in the original structure to exclude   | False      | List(Ident)             | `omit(field1, field2, ...)`       |
| derive          | Things to derive on the newly generated struct     | False      | List(Path)              | `derive(Debug, thiserror::Error)` |
| preset          | Behaviours and/or defaults to apply                | False      | none/write/read         | `preset = "all"`                  |
| attributes_with | Attributes to inherit at both struct & field level | False      | none/oai/deriveless/all | `attributes_with = "all"`         |


### Example
```rust
#[derive(Clone, restructed::Models)]
#[model(base(derive(Debug)))] // All models now *MUST* derive Debug (despite parent)
#[view(UserView)]
#[patch(UserPatch)]
struct User {
    id: i32,
    display_name: String,
    bio: String,
    extra: Option<String>,
    password: String,
}

fn debug_models() {
  let user = User {
    id: 1,
    display_name: "Dude".to_string(),
    bio: "Too long didn't read".to_string(),
    extra: None,
    password: "ezpz".to_string(),
  };

  let view: UserView = user.clone().into(); // Automatically gen from model
  print!("A view of a user {:?}", view);

  let patch: UserPatch = user.clone().into(); // Automatically gen from model
  print!("A patch of a user {:?}", patch);
}
```


<br/>

# Argument Behaviours

## `preset` 
A _string literal_ of the preset to use, presets are a set of defaults to apply to a model. *Below is a list of what arguments are composed in a preset.* [e.g. `preset = "none"`] 
  - **none** - Does nothing and is the default behaviour [**Default**]
  - **write** *['openapi' Feature Flag]* - Designed to only show properties that can be written to.
    - `omit` - Applied as a base, any fields with `#[oai(read_only)]` attribute are removed, your fields/omit is applied after
    - `option` **patch only** - Arg defaults to `MaybeUndefined`

  - **read** *['openapi' Feature Flag]* - Designed to only show properties that can always be read.
    - `omit` - Applied as a base, any fields with `#[oai(write_only)]` attribute are removed, your fields/omit is applied after
    - `option` **patch only** - arg defaults to `MaybeUndefined`

## `attributes_with`
A _string literal_ of the attributes to inherit at both struct & field level. *Below is a list of values.* [e.g. `attributes_with = "none"`] 
  - **none** - Does not Includes any attributes [**Default**]
  - **oai** *['openapi' Feature Flag]* - Includes all Poem's OpenAPI attributes
  - **deriveless** - Includes all attributes but omits the derive attributes
  - **all** - Includes all attributes

# Known Limitations

- *Generic Structs & Enums* - At the moment, this crate **doesn't support** deriving models on Structs that need to be generic (e.g. deriving on a `Struct<T>`). I just don't need the feature, contributions are welcome however!

<br/>

# Crate Features
Links are to other crates GitHub page that are related to the features.<br/>

## Poem OpenAPI
Enables wrapping `Option<T>` from the source struct with `MaybeUndefined<T>` from the [poem-openapi](https://github.com/poem-web/poem/tree/master/poem-openapi) crate in `patch` models. All `oai(...)` attributes can also be explictly copied over to the generated struct meaning you keep all validators, etc..

```rust 
use restructed::Models;

#[derive(poem_openapi::Object, Models)]
#[oai(skip_serializing_if_is_none, rename_all = "camelCase")]
#[model(base(derive(poem_openapi::Object, Debug)), defaults(preset = "read"))]
#[patch(UserUpdate, preset = "write")]
#[view(UserProfile)]
#[view(UserNames, fields(username, name, surname))]
pub struct User {
    #[oai(read_only)]
    pub id: u32,
    // profile
    #[oai(validator(min_length = 3, max_length = 16, pattern = r"^[a-zA-Z0-9_]*$"))] // oai attributes carry over with `preset = write/write` or attributes_with="oai"
    pub username: String,
    #[oai(validator(min_length = 5, max_length = 1024), write_only)]
    pub password: String,
    #[oai(validator(min_length = 2, max_length = 16, pattern = r"^[a-zA-Z\s]*$"))]
    pub name: Option<String>,
    #[oai(validator(min_length = 2, max_length = 16, pattern = r"^[a-zA-Z\s]*$"))]
    pub surname: Option<String>, // in patch modeels, this is `MaybeUndefined` type with default with preset `read` or `write` (or option = MaybeUndefined)
    #[oai(read_only)]
    pub joined: u64,
}
```
