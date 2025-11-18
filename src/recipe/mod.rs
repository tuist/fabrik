// Portable recipes - Cross-platform JavaScript automation
//
// This module provides cross-platform build automation using JavaScript that runs
// on Fabrik's embedded QuickJS runtime with custom Fabrik APIs exposed.

pub mod executor;
pub mod runtime;

pub use executor::RecipeExecutor;
