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

# Usage

This library requires the `nightly` channel.

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
use restructed::Models;

#[derive(restructed::Models)]
struct User {
    id: i32,
    username: String
}

```

And then add attributes for each model you want to create.

```rust
#[derive(restructed::Models)]
#[patch(UserUpdatables, omit(id))] // <-- Wraps all fields in a new struct with Option
#[view(UserId, fields(id))]        // <-- Selectively includes fields in a new struct
struct User {
    id: i32,
    username: String,
}
```

Continue reading for the available models and their breakdown.

Now anywhere that requires a `T: OrderStoreFilter` will also accept an `&T` or `Arc<T>` where `T` implements `OrderStoreFilter`.

## Models

Each model is defined using an attribute after deriving `Models` and multiple models (of the same kind) can be had with multiple attributes.

### `view`

A selective subset of fields from the original model of the same types.

**Arguements:**

- `name` - The name of the struct the generate (**Required**, **Must be first** e.g. `MyStruct`)
- `fields` - A _list_ of field names in the original structure to carry over (**Required**, e.g. `fields(field1, field2, ...)`)
- `derive` - A _list_ of derivables (in scope) to derive on the generated struct (e.g. `derive(Clone, Debug, thiserror::Error)`)
- `default_derives` - A *bool*, if `true` *(default)* then the a list of derives will be additionally derived. Otherwise, `false` to avoid this (e.g. `default_derives = false`)

**Example:**
```rust
   // Original
   #[derive(restructed::Models)]
   #[view(UserProfile, fields(display_name, bio), derive(Clone), default_derives = false)]
   struct User {
       id: i32,
       display_name: String,
       bio: String,
       password: String,
   }
```
Generates:
```rust
   #[derive(Clone)]
   struct UserProfile {
       display_name: String,
       bio: String,
   }
```

# patch
A complete subset of fields of the original model wrapped in `Option<T>` with the ability to omit instead select fields.

**Arguements:**
- `name` - The name of the struct the generate (**Required**, **Must be first** e.g. `MyStruct`)
- `omit` - A *list* of field names in the original structure to omit (**Required**, e.g. `fields(field1, field2, ...)`)
- `derive` - A *list* of derivables (in scope) to derive on the generated struct (e.g. `derive(Clone, Debug, thiserror::Error)`)
- `default_derives` - A *bool*, if `true` *(default)* then the a list of derives will be additionally derived. Otherwise, `false` to avoid this (e.g. `default_derives = false`)
 
**Example:**
```rust
   // Original
   #[derive(restructed::Models)]
   #[patch(UserUpdate, omit(id))]
   struct User {
      id: i32,
      display_name: String,
      bio: String,
      password: String,
   }
```
 
Generates:
```rust
   #[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)] // <-- Default derives (when *not* disabled)
   struct UserUpdate {
       display_name: Option<String>,
       bio: Option<String>, // MaybeUndefined<String> with feature 'openapi'
       password: Option<String>,
   }
```

## Crate Features

Links are to other crates GitHub page that are related to the features.<br/>
Only `builder` is enabled by default.

### `openapi`

Wraps `Option<T>` from the source struct with `MaybeUndefined<T>` from the [poem-openapi](https://github.com/poem-web/poem/tree/master/poem-openapi) crate in `patch` models. All `oai(...)` attributes are also copied over to the generated struct meaning you keep all validators, etc..

### `builder`

Uses the [typed-builder](https://github.com/idanarye/rust-typed-builder) crate to derive a builder for add a type safe builder for all generated models.

### `welds`

Generates a function to merge changes for returning a `DbState<T>` from the [welds](https://github.com/weldsorm/welds) crate.

## Contributions & Bugs

This is my first self publish proc macro so any feedback and feature request, changes, pull requests are all welcome! <br/>
If you find any bugs do submit a github issue with any relavent information and I'll try to fix it.
