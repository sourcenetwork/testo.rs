pub mod action;
pub mod multiplier;

pub mod derive {
    pub use testo_derive::Action;
    pub use testo_derive::Multiplier;
}

pub mod prelude {
    pub use crate::action::{Action, ActionInstance, IntoAsync};
    pub use crate::multiplier::{Multiplier};
}
