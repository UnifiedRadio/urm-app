// editcp bridge adapter — Phase 6
// Reads DMR codeplugs via the `editcp` CLI subprocess (GPL-2.0-or-later).
// Write not implemented: URC→editcp conversion requires further work.
// Reference: https://github.com/DaleFarnsworth-DMR/editcp

mod bridge;
mod editcp_env;

pub use bridge::EditcpAdapter;
