//! The bezique rules engine.
//!
//! The rules are those of `docs/RULES.md` (Foster, 1897); the reasoning behind
//! the structure is in `docs/DESIGN.md`.

pub mod agents;
pub mod cards;
pub mod deal;
pub mod game;
pub mod hand;
pub mod melds;
pub mod observation;
pub mod rng;
pub mod tournament;
pub mod tricks;
pub mod words;
