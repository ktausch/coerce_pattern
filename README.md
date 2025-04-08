# coerce_pattern

`coerce_pattern` is a Rust library crate ([here](https://crates.io/crates/coerce_pattern) on crates.io) containing macros that force expressions into patterns. It is version-controlled on GitHub [here](https://github.com/ktausch/coerce_pattern).

This crate contains two proc macros relating to asserting, under penalty of panic, that expressions match patterns.

1. `coerce_pattern!` matches an expression to a target pattern and evaluates to an expression of the inner variables from the pattern, panicking if the pattern doesn't match
2. `assert_pattern!` matches an expression to a target and panics if the pattern doesn't match.

To use them, run the following command in your project directory.

```bash
cargo add coerce_pattern
```

Then, you can use the macros using

```rust
use coerce_pattern::{coerce_pattern, assert_pattern};
```

## `assert_pattern!` macro

The `assert_pattern!($expression, $target_pattern)` macro roughly expands out to

```rust
match $expression {
    $target_pattern => {}
    _ => panic!(),
}
```

This means you can use it whenever you want to claim, at risk of panicking (such as in unit tests in particular), that a particular expression matches a particular pattern. For example

```rust
let x = Some(Some(1));
assert_pattern!(x, Some(Some(_)));
```

## `coerce_pattern!` macro

The `coerce_pattern!($expression, $target_pattern, $result)` macro roughly expands out to

```rust
match $expression {
    $target_pattern => $result,
    _ => panic!(),
}
```

This is useful in cases similar to `assert_pattern!`, except that you actually want to capture one or more variables from the matching pattern. Consider

```rust
struct LegalEntity {
    Person { name: String },
    Company { dba: String, states: Vec<String> },
}
let entity = LegalEntity::Company{
    dba: String::from("my_company"),
    states: ["NY", "NJ", "CT"].into_iter().map(String::from).collect()
}
let states = coerce_pattern!(entity, LegalEntity::Company { states, .. }, states);
assert_eq!(states.len(), 3);
```

See the documentation in `src/lib.rs` for more specific details and more examples of each macro's usage.
