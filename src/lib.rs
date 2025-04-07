//! This crate contains two proc macros relating to asserting, under
//! penalty of panic, that expressions match patterns.
//!
//! 1. `coerce_pattern!` matches an expression to a target pattern and evaluates to
//! an expression of the inner variables from the pattern, panicking if the pattern
//! doesn't match
//! 2. `assert_pattern!` matches an expression to a target and panics if the pattern
//! doesn't match.
//!
//! To use them, run the following command in your project directory.
//! ```bash
//! cargo add coerce_pattern
//! ```
//! Then, you can use the macros using
//! ```rust
//! use coerce_pattern::{coerce_pattern, assert_pattern};
//! ```
//!
//! See below for detailed documentation on the macros.
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Token,
};

/// Full input of the `coerce_pattern!` macro
struct CoercePatternInput {
    /// expression that should be coerced into a pattern
    expression: Expression,
    /// pattern that expression should be coerced into
    target: Target,
    /// expression (that is valid given target pattern
    /// and surrounding context) that should be returned
    result: Expression,
}

impl Parse for CoercePatternInput {
    /// Parses the input of coerce_pattern! by separating it
    /// into expression, target, and result in that order
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expression = input.parse()?;
        input.parse::<Token![,]>()?;
        let target = input.parse()?;
        input.parse::<Token![,]>()?;
        let result = input.parse()?;
        Ok(Self {
            expression,
            target,
            result,
        })
    }
}

impl ToTokens for CoercePatternInput {
    /// Performs code-generation for the coerce_pattern! macro. Uses a
    /// match with one arm with target pattern and one wildcard arm.
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            expression,
            target,
            result,
        } = self;
        tokens.extend(quote! {
            match #expression {
                #target => { #result }
                _ => panic!("expression didn't match target pattern in coerce_pattern")
            }
        });
    }
}

/// Full input of the `assert_pattern!` macro
struct AssertPatternInput {
    /// expression that should match the pattern
    expression: Expression,
    /// pattern that the expression should match
    target: Target,
}

impl Parse for AssertPatternInput {
    /// Parses the input of the assert_pattern! macro by parsing
    /// the expression and target pattern in that order.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expression = input.parse()?;
        input.parse::<Token![,]>()?;
        let target = input.parse()?;
        Ok(Self { expression, target })
    }
}

impl ToTokens for AssertPatternInput {
    /// Performs code-generation for the assert_pattern! macro. Uses a
    /// match with one arm with target pattern and one wildcard arm.
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { expression, target } = self;
        tokens.extend(quote! {
            match #expression {
                #target => {}
                _ => panic!("expression didn't match target pattern in assert_pattern")
            }
        });
    }
}

/// The target pattern can be any pattern (including refutable patterns)
struct Target(syn::Pat);

impl Parse for Target {
    /// Parses the Target pattern using syn::Pat::parse_multi because
    /// it can accept any pattern that can label a match arm
    fn parse(input: ParseStream) -> syn::Result<Self> {
        syn::Pat::parse_multi(input).map(Self)
    }
}

impl ToTokens for Target {
    /// Code-generation of a target is the same as the underlying syn::Pat
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

/// Expression is a thin wrapper around syn::Expr
struct Expression(syn::Expr);

impl Parse for Expression {
    /// Parses input in the same way as the underlying syn::Expr
    fn parse(input: ParseStream) -> syn::Result<Self> {
        syn::Expr::parse(input).map(Self)
    }
}

impl ToTokens for Expression {
    /// Code-generation of an expression is the same as the underlying syn::Expr
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

/// Asserts that an expression matches a pattern, like a
/// generalized `assert!(result.is_ok())`.
///
/// `assert_pattern!($e, $p)` expands roughly to
/// ```text
/// match $e {
///     $p => {}
///     _ => panic!()
/// }
/// ```
///
/// # Motivation
///
/// One of the main motivations of writing this library,and
/// `assert_pattern!` in particular, is to make tests more concise.
/// There are many cases where a function should return an object
/// matching a specifically general pattern, but, as this pattern becomes
/// more complicated, so does the unit test, but this doesn't need to be
/// the case. Compare the following two blocks of code. First, the
/// version without using `assert_pattern!`
/// ```rust
/// # fn rand_bool() -> bool {true}
/// struct S {
///     x: u32,
///     y: u32,
/// }
/// fn one_of_two_s_forms(which: bool) -> S {
///     if which {
///         S {x: 1, y: 2}
///     } else {
///         S {x: 3, y: 4}
///     }
/// }
/// let s = one_of_two_s_forms(rand_bool());
/// assert!(((s.x == 1) && (s.y == 2)) || ((s.x == 3) && (s.y == 4)));
/// ```
/// Next, the same code using `assert_pattern!`
/// ```rust
/// # use coerce_pattern::assert_pattern;
/// # fn rand_bool() -> bool {true}
/// struct S {
///     x: u32,
///     y: u32,
/// }
/// fn one_of_two_s_forms(which: bool) -> S {
///     if which {
///         S {x: 1, y: 2}
///     } else {
///         S {x: 3, y: 4}
///     }
/// }
/// assert_pattern!(one_of_two_s_forms(rand_bool()), S{x: 1, y: 2} | S{x: 3, y: 4});
/// ```
///
///
/// # Option example
///
/// One way of using `assert_pattern!` is to destructure an object
/// (like a tuple here) inside an Option when you would otherwise use
/// `unwrap` and a match statement, e.g.
/// ```rust
/// # use coerce_pattern::assert_pattern;
/// let o = Some((1, "this string could change and this code still wouldn't panic"));
/// assert_pattern!(o, Some((1, _)));
/// ```
/// This code is roughly equivalent to
/// ```rust
/// let o = Some((1, "this string could change and this code still wouldn't panic"));
/// assert!(
///     match o {
///         Some((1, _)) => true,
///         _ => false,
///     }
/// )
/// ```
///
/// # Custom type example
///
/// More useful examples arise naturally in cases involving custom types
/// ```rust
/// # use coerce_pattern::assert_pattern;
/// enum MyEnum {
///     A(u32),
///     B(i64),
/// }
/// let e = MyEnum::B(-1);
/// assert_pattern!(e, MyEnum::B(_));
/// ```
/// This code will panic if `e` is set to a `MyEnum::A`. If it doesn't panic,
/// though, then `x` is bound to the i64 in the `MyEnum::B` instance.
#[proc_macro]
pub fn assert_pattern(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let i = parse_macro_input!(input as AssertPatternInput);
    quote! { #i }.into()
}

/// Coerces an expression into a pattern, like a generalized unwrap.
///
/// `coerce_pattern!($e, $t, $r)` expands roughly to
/// ```text
/// match $e {
///     $t => $r,
///     _ => panic!()
/// }
/// ```
///
/// Note that, up to differences in panic messages, `assert_pattern!($e, $t)` is
/// equivalent to `coerce_pattern!($e, $t, {})`.
///
/// # Motivation
///
/// One of the main uses of this macro is to better measure test coverage in
/// library code which contains panics. For example, consider the following
/// code snippet:
/// ```rust
/// enum MyEnum {
///     A { x: u32 },
///     B(u64, u64),
/// }
/// impl MyEnum {
///     fn new_b(first: u64) -> Self {
///         Self::B(first, 0)
///     }
/// }
/// let o = MyEnum::new_b(763); // guaranteed to be a MyEnum::B
/// let x = match o {
///     MyEnum::B(_, x) => x,
///     MyEnum::A{..} => panic!("this panic will never be tested or testable"),
/// };
/// assert_eq!(x, 0);
/// ```
/// This code will lead to a line of untested code, no matter how thoroughly it is tested.
/// Compare this to the same code using `coerce_pattern!`
/// ```rust
/// # use coerce_pattern::coerce_pattern;
/// enum MyEnum {
///     A { x: u32 },
///     B(u64, u64),
/// }
/// impl MyEnum {
///     fn new_b(first: u64) -> Self {
///         Self::B(first, 0)
///     }
/// }
/// let o = MyEnum::new_b(763); // guaranteed to be a MyEnum::B
/// let x = coerce_pattern!(o, MyEnum::B(_, x), x);
/// assert_eq!(x, 0);
/// ```
/// In contrast to the code using `match ... { ... panic!()}`,
/// this code has no lines or regions that aren't tested.
///
/// # Option example
///
/// A trivial example (probably better replaced by `Option::unwrap()`)
/// that unwraps an option while also performing an expression.
/// ```rust
/// # use coerce_pattern::coerce_pattern;
/// let o = Some(1);
/// let x = coerce_pattern!(o, Some(y), y + 2);
/// assert_eq!(x, 3);
/// ```
/// Note that this is probably better replaced with `let x = o.unwrap() + 2;`
/// The only difference between the two representations is the panic message.
///
/// # Custom type example
///
/// More useful examples arise naturally in cases involving custom types
/// ```rust
/// # use coerce_pattern::coerce_pattern;
/// enum MyEnum {
///     A(u32),
///     B(i64),
/// }
/// let e = MyEnum::B(-1);
/// let x = coerce_pattern!(e, MyEnum::B(y), y);
/// assert_eq!(x, -1);
/// ```
/// This code will panic if `e` is set to a `MyEnum::A`. If it doesn't panic,
/// though, then `x` is bound to the i64 in the `MyEnum::B` instance.
#[proc_macro]
pub fn coerce_pattern(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let i = parse_macro_input!(input as CoercePatternInput);
    quote! { #i }.into()
}
