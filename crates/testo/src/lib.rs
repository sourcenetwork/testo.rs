/*!
This crate contains a shared integration testing framework for Source projects.

Tests are defined using strongly typed, declarative, [Actions](action::Action).

[Multipliers](multiplier::Multiplier) may be configured to mutate the declared actions to scale with various project-specific complexity multipliers.  These may
be selected and/or excluded based on the location, environment, and test actions.

The Rust implementation of Testo is new, and currently lacks large-scale example projects.  The tests within this repository, particularly in
[full.rs](https://github.com/sourcenetwork/testo.rs/blob/main/testo/tests/full.rs) may be useful in the near term. Alternatively, the
[Go implementation](https://github.com/sourcenetwork/testo) is built off of the same concepts, and is best illustrated via
[CoreKV](https://github.com/sourcenetwork/corekv/tree/main/test).
*/

/// The core elements needed to define Testo tests.
pub mod action;

/// The core framework for defining complexity multipliers, allowing existing [Action](action::Action) based tests to be used as the base for
/// property based testing.
pub mod multiplier;

/// Convenience derive-macros that can optionally be used to eliminate some of the boilerplate.
pub mod derive {
    pub use testo_derive::Action;
    pub use testo_derive::Multiplier;
}

/// Convenience re-exports of core items.
pub mod prelude {
    pub use crate::action::{Action, ActionInstance, IntoAsync};
    pub use crate::multiplier::Multiplier;
}
