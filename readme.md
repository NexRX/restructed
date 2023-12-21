# `restructed` 

A quick and easy way to create derivative models of your existing types without repeating yourself all the damn time.

# Usage

This library requires the `nightly` channel.

Add `restructed` to your projects `Cargo.toml`:

```toml
restructed = "0.1"
```

Add the import and derive it on the target struct

```rust
use restructed::Models;

#[derive(Models)]
struct User {
    id: i32,
    username: String
}

```

And then add attributes for each model you want to create.

```rust
#[derive(Models)]
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
A selective subset of fields of the original model. 

**Arguements:**
- `name` - The name of the struct the generate (**Required**, **Must be first** e.g. `MyStruct`)
- `fields` - A *list* of field names in the original structure to carry over (**Required**, e.g. `fields(field1, field2, ...)`)
- `derive` - A *list* of derivables (in scope) to derive on the generated struct (e.g. `derive(Clone, Debug, thiserror::Error)`)
- `derive_defaults` - A *bool*, if `true` *(default)* then the a list of derives will be additionally derived. Otherwise, `false` to avoid this (e.g. `derive_defaults = false`)

