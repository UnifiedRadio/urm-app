// qDMR bridge adapter — Phase 6
// Reads/writes DMR codeplugs via the `qdmr` CLI subprocess (GPL-3.0-or-later).
// Reference: https://github.com/hmatuschek/qdmr

mod bridge;
mod converter;
mod qdmr_env;

pub use bridge::QdmrAdapter;
