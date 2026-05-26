// dmrconfig bridge adapter — Phase 6
// Reads DMR codeplugs via the `dmrconfig` CLI subprocess (BSD-3-Clause).
// Write not implemented: URC→dmrconfig binary conversion requires further work.
// Reference: https://github.com/OpenRTX/dmrconfig

mod bridge;
mod dmrconfig_env;

pub use bridge::DmrconfigAdapter;
