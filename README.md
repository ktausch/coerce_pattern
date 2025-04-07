# coerce_pattern

`coerce_pattern` is a Rust library crate containing macros that force expressions into patterns. It is version-controlled on GitHub [here](https://github.com/ktausch/coerce_pattern).

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

See the documentation in `src/lib.rs` for more details
