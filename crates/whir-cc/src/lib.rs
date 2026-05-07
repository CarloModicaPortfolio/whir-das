//! `whir-cc` — Erasure-Code Commitment from the WHIR IOPP-of-proximity.
//!
//! Realises the four-function `CC = (Setup, Com, Open, Ver)` interface of
//! Foundations-of-DAS Def. 8 by wrapping a forked `whir-p3` and discharging
//! position- and code-binding at the Johnson list-decoding radius.
//!
//! See `README.md` at the workspace root.

#![doc(html_root_url = "https://docs.rs/whir-cc/0.0.0")]
