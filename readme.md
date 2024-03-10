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


## Common Arguments
These are arguments that can be applied to all models.

- `name` - The name of the struct the generate  [**Required**, **Must be first** e.g. `MyStruct`]
- `fields` [**Only One**]
  - `fields` - A _list_ of field names in the original structure to carry over  [e.g. `fields(field1, field2, ...)`]
  - `omit` - A _list_ of field names in the original structure to omit  [e.g. `omit(field1, field2, ...)`]
- `derive` - A _list_ of derivables to derive on the generated struct just like you would normally [e.g. `derive(Clone, Debug, thiserror::Error)`]
- `preset` - A _string literal_ of the preset to use, presets are a set of defaults to apply to a model. *Below is a list of what arguments are composed in a preset.* [e.g. `preset = "none"`] 
  - **none** - **Default**, does nothing and is the default behaviour
  - **write** *['openapi' Feature Flag]* - Designed to only show properties that can be written to.
    - `omit` - Applied as a base, any fields with `#[oai(read_only)]` attribute are removed, your fields/omit is applied after
    - `option` **patch only** - Arg defaults to `MaybeUndefined`

  - **read** *['openapi' Feature Flag]* - Designed to only show properties that can always be read.
    - `omit` - Applied as a base, any fields with `#[oai(write_only)]` attribute are removed, your fields/omit is applied after
    - `option` **patch only** - arg defaults to `MaybeUndefined`

- `attributes_with` - A _string literal_ of the attributes to inherit at both struct & field level. *Below is a list of values.* [e.g. `attributes_with = "none"`] 
  - **none** - Does not consider any attributes [**Default**]
  - **oai** *['openapi' Feature Flag]* - Composes all Poem's OpenAPI attributes
  - **deriveless** - Composes all attributes but omits the derive attributes
  - **all** - Composes all attributes

<br/>

# Models
Derivable models via the struct attributes.

## **Patch**
A subset model where every field's type is an option in some way. It's called patch because it reflect a REST / DB patch of the original struct.

#### **Unique Args**
- `option` - A _Identifer_ that allows a different option implementation from supported crates [e.g. `option = MaybeUndefined` (from poem-openapi)]
  - **Option** - The standard implementation of Option [**Default**]
  - **MaybeUndefined** *['openapi' Feature Flag]* - Use Poem's OpenAPI crate and it's Option implmentation

#### **Example**
```rust
#[derive(restructed::Models)]
#[patch(UserUpdate, omit(id), option = Option)]
struct User {
    id: i32,
    display_name: String,
    bio: String,
    extra: Option<String>,
    password: String,
}
```

will expand into something like this:
```rust
struct UserUpdate {
    display_name: Option<String>,
    bio: Option<String>,
    extra: Option<Option<String>>,
    password: Option<String>,
}
```

## **View**
A simple subset of the deriving model/struct. 

#### **Unique Args**
- N/A

#### **Example**
```rust
#[derive(restructed::Models)]
#[view(UserPublic, fields(display_name, bio))]
struct User {
    id: i32,
    display_name: String,
    bio: String,
    extra: Option<String>,
    password: String,
}
```

will expand into something like this:
```rust
struct UserPublic {
    display_name: String,
    bio: String,
}
```

<br/>

# Complex Example
Just to demonstrate the versitility of this crate, here is an example using all the possible arguments at once using all features.

## Poem OpenAPI 
Each attribute is copied over so all your validations are kept

```rust 
use restructed::Models;

#[cfg(test)] // For rust_doc test purposes
#[derive(poem::Object, Models)]
#[oai(skip_serializing_if_is_none, rename_all = "camelCase")]
#[patch(UserUpdate, preset = "write", derive(Object))]
#[view(UserProfile, preset = "view", derive(Object))]
#[view(UserNames, preset = "view", derive(Object))]
pub struct User {
    #[oai(read_only)]
    pub id: u32,
    // profile
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

<br/>

# Crate Features
Links are to other crates GitHub page that are related to the features.<br/>

## Poem OpenAPI
Enables wrapping `Option<T>` from the source struct with `MaybeUndefined<T>` from the [poem-openapi](https://github.com/poem-web/poem/tree/master/poem-openapi) crate in `patch` models. All `oai(...)` attributes can also be explictly copied over to the generated struct meaning you keep all validators, etc..