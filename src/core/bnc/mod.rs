pub mod config;

/// Holds implementation of async rest API fetcher using reqwest crate;
/// Implements needed traits described in the fetch module.
pub mod rest;

/// Traits for various(and used) parts of bnc api.
pub mod snapshot;

/// Hold BNC type and entities definitions that are in use in current application.
///
/// Not all the deserializable traits are included here, some are moved to specific submodules, like snapshot module.
pub mod data;

/// Holds error and result definitions for this part of the core.
pub mod error;

/// Holds realtime interactions with BNC API.
pub mod ws;
