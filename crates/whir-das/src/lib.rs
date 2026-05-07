//! `whir-das` — Data Availability Sampling on top of `whir-cc`.
//!
//! Realises the HSW Def. 1 DAS primitive `(Setup, Encode, V1, V2, Ext)` by
//! instantiating the generic Foundations-of-DAS §6.3 compiler `DAS[CC, Sample]`
//! with `whir-cc` as the erasure-code commitment and a pluggable index sampler.
//!
//! See `README.md` at the workspace root.

#![doc(html_root_url = "https://docs.rs/whir-das/0.0.0")]
