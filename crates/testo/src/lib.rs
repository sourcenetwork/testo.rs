/*!
This crate contains a shared integration testing framework for Source projects.

Tests are defined using strongly typed, declarative, [Actions](action::Action).

[Multipliers](multiplier::Multiplier) may be configured to mutate the declared actions to scale with various project-specific complexity multipliers.  These may
be selected and/or excluded based on the location, environment, and test actions.

The Rust implementation of Testo is new, and currently lacks large-scale example projects.  The tests within this repository, particularly in
[full.rs](https://github.com/sourcenetwork/testo.rs/blob/main/testo/testo/tests/full.rs) may be useful in the near term. Alternatively, the
[Go implementation](https://github.com/sourcenetwork/testo) is built off of the same concepts, and is best illustrated via
[CoreKV](https://github.com/sourcenetwork/corekv/tree/main/test).
*/

/// The core elements needed to define Testo tests.
pub mod action;

/// The core framework for defining complexity multipliers, allowing existing [Action](action::Action) based tests to be used as the base for
/// property based testing.
pub mod multiplier;

pub mod executor;
pub mod executor_tpa;
pub mod executor_multi;

/// Convenience derive-macros that can optionally be used to eliminate some of the boilerplate.
pub mod derive {
    /// Implements [Action](crate::action::Action) for the enum that this attribute sits upon.
    ///
    /// It requires an additional parameter specifying the exact name of the actions' [TState](crate::action::Action::TState) type
    /// as a string.
    ///
    /// In the short-term it also requires that each enum's variant matches the type-name of the [Action](crate::action::Action)
    /// that it contains.
    ///
    /// Its usage is completely optional, and is currently quite badly coded - using it in unexpected ways might result in unhelpful
    /// compiler errors or unwanted behaviour.
    ///
    /// ## Example
    /// ```
    /// # use testo::prelude::*;
    /// # use testo::derive::Action;
    /// #
    /// # #[derive(Clone, Debug)]
    /// # struct ExampleAction{}
    /// #
    ///   struct State {
    ///     // ... state props
    ///   }
    ///
    ///   impl Action for ExampleAction {
    ///     type TState = State;
    ///     fn execute(&self, state: &mut State){}
    ///   }
    ///
    ///   #[derive(Clone, Debug, Action)]
    ///   #[TState = "State"]
    ///   enum Actions {
    ///     ExampleAction(ExampleAction),
    ///   }
    /// ```
    pub use testo_derive::Action;

    /// Implements [Multiplier](crate::multiplier::Multiplier) for the enum that this attribute sits upon.
    ///
    /// It requires an additional parameter specifying the exact name of the actions' enum as a string.
    ///
    /// In the short-term it also requires that each enum's variant matches the type-name of the [Multiplier](crate::multiplier::Multiplier)
    /// that it contains.
    ///
    /// Its usage is completely optional, and is currently quite badly coded - using it in unexpected ways might result in unhelpful
    /// compiler errors or unwanted behaviour.
    ///
    /// ## Example
    /// ```
    /// # use testo::prelude::*;
    /// # use testo::derive::Multiplier;
    /// # use testo::derive::Action;
    /// #
    /// # #[derive(Clone, Debug)]
    /// # struct ExampleAction{}
    /// #
    ///   struct ExampleMultiplier{}
    /// #
    /// # impl Multiplier<Actions> for ExampleMultiplier {
    /// #  const NAME: &str = "ExampleMultiplier";
    /// #
    /// #  fn apply(
    /// #       &self,
    /// #      actions: &[ActionInstance<Actions>],
    /// #  ) -> Vec<ActionInstance<Actions>> { todo!() }
    /// # }
    ///
    ///   struct State {
    ///       // ... state props
    ///   }
    /// #
    /// # impl Action for ExampleAction {
    /// #     type TState = State;
    /// #     fn execute(&self, state: &mut State){}
    /// # }
    ///
    ///   #[derive(Clone, Debug, Action)]
    ///   #[TState = "State"]
    ///   enum Actions {
    ///       ExampleAction(ExampleAction),
    ///   }
    ///
    ///   #[derive(Multiplier)]
    ///   #[TActionAsync = "Actions"]
    ///   enum Multipliers {
    ///       ExampleMultiplier(ExampleMultiplier),
    ///   }
    /// ```
    pub use testo_derive::Multiplier;
}

/// Convenience re-exports of core items.
pub mod prelude {
    pub use crate::action::{Action, ActionInstance, IntoAsync};
    pub use crate::multiplier::Multiplier;
}
