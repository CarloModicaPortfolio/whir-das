# WHIR-DAS

> **Status:** boilerplate / pre-implementation. Theorems and code are work in progress; this repo currently exposes only the workspace skeleton and the design rationale below.

A reference Rust implementation of **WHIR-DAS**: a transparent, post-quantum Data Availability Sampling scheme built by plugging the [WHIR](https://eprint.iacr.org/2024/1586) IOPP-of-proximity into the [Foundations-of-DAS (HSW'23)](https://eprint.iacr.org/2023/1079) compiler, via the opening-consistency criterion of [FRIDA](https://eprint.iacr.org/2024/248).

The repo ships two crates:

- [`whir-cc`](crates/whir-cc) — the *Erasure Code Commitment* scheme: a thin wrapper that packages a forked [`whir-p3`](https://github.com/tcoratger/whir-p3) as the four-function `CC = (Setup, Com, Open, Ver)` interface of HSW Def. 8.
- [`whir-das`](crates/whir-das) — the *Data Availability Sampling* primitive `(Setup, Encode, V1, V2, Ext)` from HSW Def. 1, instantiated by feeding `whir-cc` into the generic HSW §6.3 `DAS[CC, Sample]` compiler.

A minimal-surface fork of `whir-p3` (additive, no Fiat–Shamir reshuffle, no existing-caller breakage) sits underneath. Upstreaming the fork is post-v1.

## Build

```sh
cargo build --workspace
cargo test  --workspace
```

## TL;DR

WHIR is a multilinear, hash-based IOPP-of-proximity. The PCS constructed upon it powers the leanMultisig effort, for the PQ-consensus signature aggregation.

The Lean Ethereum "Data Layer" milestone — hash-based DAS commitments replacing KZG — has no assigned cryptographic substrate. This project plugs WHIR into the Foundations-of-DAS compiler, producing a transparent, post-quantum DAS scheme.

The work splits into:

1. **Theory.** Showing WHIR satisfies the FRIDA opening-consistency criterion at the Johnson list-decoding radius, and a concrete construction of a full DAS scheme based on WHIR.
2. **A minimal-surface fork of `whir-p3`** (Plonky3 crate) exposing the two entry points an erasure-code commitment needs.
3. **v1 reference implementation** of the protocol, in the two crates above.

## Problem

Ethereum's PQ roadmap (strawmap.org M*, Lean Ethereum) commits to replacing KZG-based DAS commitments with hash-based, post-quantum commitments. The cryptographic substrate is unassigned.

The published transparent, post-quantum, IOPP-based instantiations of the Foundations-of-DAS framework all build on FRI:

1. **FRIDA** (Hall-Andersen, Simkin, Wagner — CRYPTO 2024): first FRI-based, transparent DAS; instantiated within the unique-decoding radius.
2. **Deep-FRIDA** (Yang–Zhang–Deng, 2026): DEEP technique lifts FRIDA to the Johnson list-decoding radius, 1.2–1.8× smaller commitments.
3. **FRIVail** (Srivastava / Avail, 2025): per-row FRI-Binius proofs with three aggregation strategies.

WHIR is the natural successor to FRI. Same hash-based, RS-proximity skeleton, with three structural upgrades that all matter for DAS:

1. **Multilinear-native code**: built on the constrained Reed–Solomon (CRS) which expresses 2D / multidimensional layouts and partial-evaluation queries inside the proximity test rather than via external gluing.
2. **STIR-style rate-decreasing iterations**: compound into smaller aggregate proximity proofs.
3. **Concretely cheaper verifier**: at d=2²⁴, ρ=1/2, 128-bit: ~2× fewer hashes (2.7k vs 5.6k), ~2× smaller arguments, ~4× faster verifier than FRI, at competitive prover time.

On the DAS-relevant verifier dimensions — aggregate proximity-proof size, sampler-side hash cost, and native support for multidimensional layouts — WHIR Pareto-dominates the FRI line.

**No published paper plugs WHIR into the Foundations-of-DAS compiler.** The technical bridge is the opening-consistency criterion introduced by FRIDA (Def. 10), the property that lifts an IOPP-of-proximity to an erasure-code commitment, and from there (via the HSW compiler, §6) to a DAS scheme. Because WHIR is structurally a Reed–Solomon proximity test like FRI, the path is clear; the three new structural elements to thread through the consistency proof are (i) the per-iteration rate decrease, (ii) the CRS weight polynomial, and (iii) list-decoding-radius soundness via mutual correlated agreement (which WHIR provides natively, where Deep-FRIDA had to bolt it onto FRI via DEEP).

## Background

### What DAS is, and what Ethereum has today

Data Availability Sampling lets a light client probabilistically verify that a block's data is *available* by sampling random positions of an erasure-coded version of the data. The reconstruction guarantee is that, if enough sampler queries succeed in aggregate across the network, an honest party can later recover the original data.

Ethereum's DAS deployment is staged:

1. **EIP-4844 (Dencun, Mar 2024).** Introduced blobs with KZG commitments; every node still downloads every blob.
2. **PeerDAS / EIP-7594 (Fusaka, Dec 2025).** First actual DAS deployment. Per-blob 1D Reed–Solomon extension with cell-level KZG proofs; nodes custody and sample column subsets rather than downloading full blobs.
3. **FullDAS / Danksharding (roadmap, post-Fusaka).** 2D Reed–Solomon across the full blob matrix; same KZG substrate.
4. **Lean Ethereum mandate (Drake, Jul 2025).** Replace KZG with hash-based, post-quantum DAS commitments. Strawmap slots this at M\*; no substrate assigned.

All current and roadmap-but-pre-PQ DAS layers rely on **KZG**: trusted setup, pairing-based, **not** post-quantum.

### FRIDA and the Foundations-of-DAS line: what is provable today

The Foundations-of-DAS paper (HSW'23) builds DAS in two layers:

1. **Erasure-code commitment (CC).** A vector commitment with two binding notions tailored to coded data:
   - **Position-binding** (Def. 9): no PPT adversary opens the same position to two different values.
   - **Code-binding** (Def. 10): every set of openings is consistent with at least one codeword of the underlying erasure code.
   - **Reconstruction-binding** (Def. 11), follows from the first two (Lemma 2). The HSW compiler reduces DAS soundness to code-binding + sampler quality (Lemma 9) and DAS consistency to reconstruction-binding (Lemma 10).
2. **Generic CC → DAS compiler** (HSW §6). Given a position- and code-binding `CC` plus a sampling distribution `Sample`, the compiler produces a DAS scheme; soundness reduces to the binding properties of `CC` plus the sampler quality.

FRIDA closes the remaining gap from a low-degree test to a CC:

3. **Opening-consistency (FRIDA Def. 10).** A transcript-level property of an IOPP-of-proximity: any opening accepted by the verifier pins down the value of a fixed codeword extracted from the transcript. Informally, an honest verifier accepting an opening means it is consistent with one specific nearby codeword, not "some-or-other" nearby codeword.
4. **IOPP → ECC reduction.** FRIDA proves that any IOPP satisfying opening-consistency yields a code-binding `CC` for the underlying RS code; the resulting CC is made non-interactive via Fiat–Shamir in the ROM.
5. **FRI instantiation.** FRIDA proves FRI satisfies opening-consistency within the unique-decoding radius `δ ≤ (1−ρ)/2`, unconditionally.
6. **Deep-FRIDA extension.** Yang–Zhang–Deng (2026) lift the FRI instantiation to the Johnson radius `δ ≤ 1−√ρ` using the DEEP (Domain Extension for Eliminating Pretenders) quotient — unconditional up to Johnson; further capacity-regime gains are conjectural. Each round adds one out-of-domain opening; in exchange, fewer queries suffice for a target `2^{-λ}` and the net proof size shrinks by 1.2–1.8× over rates ρ ∈ [1/2, 1/8].

The published instantiations of this pipeline all use FRI. WHIR is the natural next entry.

### Why WHIR

WHIR is an IOPP for constrained Reed–Solomon codes, a strict generalization of plain RS that turns out to be exactly the structure DAS needs:

1. **The CRS code**. CRS[F, L, m, ŵ, σ] := { f ∈ RS[F, L, m] : Σ_{b ∈ {0,1}^m} ŵ(f̂(b), b) = σ }. A constrained RS codeword is a smooth RS codeword whose underlying multilinear f̂ additionally satisfies a sumcheck-style identity specified by the weight polynomial ŵ and target σ.
2. **Trivial-weight regime**: WHIR is a plain low-degree test. Setting ŵ ≡ 0, σ = 0 makes the constraint vacuous and CRS = RS. WHIR collapses to a hash-based proximity test for RS[F, L, m], exactly the IOPP shape FRIDA's compiler consumes.
3. **Nontrivial-weight regime**: multilinear-evaluation claims for free. With ŵ(Z, X) = Z · eq(ẑ, X) the constraint encodes f̂(ẑ) = σ — a multilinear evaluation claim is *inside* the proximity test, not bolted on. Multiple such claims can be batched with a random challenge.
4. **Why this matters for DAS**: A 2^{m₁} × 2^{m₂} blob layout (the FullDAS / Danksharding target) is one m-variable multilinear with m = m₁ + m₂. In WHIR an axis-aligned line probe is a partial evaluation of f̂; an off-axis or arbitrary point probe is just f̂(ẑ); both are expressed as a weight polynomial inside the same single proof.
5. **Vs the FRI line**: FRI / FRIDA are univariate. The FRIDA paper is 1D-only; any 2D-layout DAS in the FRI line would have to glue two univariate commitments externally (row-FRI + column-FRI, plus consistency checks) — and 2D-FRIDA is explicitly listed as open in FRIDA. This is the structural reason FRI-based DAS doesn't compose cleanly with multidimensional sampling, and the structural reason WHIR does.
6. **Concrete verifier cost**. At d = 2²⁴, ρ = 1/2, 128-bit security, WHIR vs FRI: ~2× smaller arguments (157 KiB vs 306 KiB), ~2× fewer verifier hashes (2.7k vs 5.6k), ~4× faster verifier (1.0 ms vs 3.9 ms), at competitive prover time.

## Thesis

WHIR is an IOPP-of-proximity for multilinear RS codes whose proximity proof is concretely smaller, whose verifier hash-count is concretely lower, and whose verifier wall-clock is concretely faster than the FRI line at every published parameter point, at competitive prover time. If WHIR satisfies the opening-consistency criterion (or a parameterised variant), it instantiates a hash-based DAS scheme that fits the strawmap M* slot at lower aggregate cost than FRIDA, with native multidimensional sampling for the FullDAS layout that FRI-based schemes can only approximate by external gluing.

Plugging WHIR into the DAS slot also unifies the cryptographic substrate across consensus and data: a single formal-verification target, a single cryptanalysis target, a single field/hash choice. This is the "Lean Craft" minimalism principle applied to PQ-cryptographic substrate.

## Theoretical scope

### `whir-cc`: an erasure-code commitment from the WHIR IOPP

The construction follows the FRIDA pipeline with WHIR replacing FRI:

1. The base oracle `f : L → F` is the L-evaluation of the data-encoded multilinear `p̂`, Merkle-committed (gives position-binding for free).
2. Run WHIR in **LDT mode** (`ŵ ≡ 0`) as the proximity test.
3. An "opening" at position `i` is the Merkle path to `f(i)` plus the WHIR transcript on the parent commitment.

The whole `whir-cc` program reduces to building one `CC = (Setup, Com, Open, Ver)` from WHIR and bounding its position- and code-binding advantages — the HSW compiler then gives DAS mechanically. The theory decomposes into one definition and four theorems:

- **Definition — opening-consistency for CRS codes.** FRIDA Def. 10 specialised to the `(F, L, m, ŵ, σ, k, t)` parameter set of constrained Reed–Solomon. The four sub-conditions — *no luck*, *bad rejected*, *suitable close*, *inconsistent rejected* — are pinned to WHIR's transcript space.
- **Theorem A — position-binding.** Trivial port of HSW Lemma 19 from Merkle CRH collision: `Adv^pos-bind ≤ Q_H² / 2^λ`.
- **Theorem B — opening-consistency.** The only genuinely new theorem of the program. Recasts WHIR's §5 soundness in Bad/Lucky shape, instantiated at the **Johnson list-decoding radius** `δ ≤ 1−√ρ`. The hardest sub-piece is *inconsistent rejected*: Merkle base-oracle binding plus a Bad/Lucky decomposition over every WHIR challenge per iteration, composed across the `n ≈ m/k` iterations. Whether per-iteration soundness composes additively or multiplicatively is the swing factor between *competitive* WHIR-DAS and *valid-but-uncompetitive* WHIR-DAS.
- **Theorem C — code-binding.** Corollary of A + B.
- **Theorem D — DAS-binding.** Corollary of HSW Lemmas 2, 9, 10.

Theorems A–D are stated and proved at the **Johnson radius** `δ ≤ 1−√ρ`, where WHIR's MCA bound is unconditional. The **capacity regime** `δ ≤ 1−ρ−ε` is conjectural under WHIR Conjecture 1 and is reported as a single footnote row.

### `whir-das`: plugging `whir-cc` into the HSW'23 compiler

Once `whir-cc` is code-binding (Theorem D), the compiler `DAS[CC, Sample]` of HSW §6.3 applies as-is. Concretely:

1. **Encode.** Encode data as a multilinear `p̂ ∈ F[X_1, …, X_m]`, evaluate on a smooth `L ⊂ F*`, Merkle-commit. Issue a single WHIR-LDT proof on the commitment that all samplers share.
2. **Sample.** Sampler queries random `ω ∈ L`, downloads `f(ω)` plus its Merkle path. Soundness bound: `Adv^sound ≤ ν(t−1, N, Q, T) + Adv^code-bind` (HSW Lemma 9).
3. **Reconstruct.** With ≥ k samples (rate `ρ = k/n`), Reed–Solomon reconstruction recovers `p̂`.

Non-free DAS-level theory:

1. **Index sampler + ν bound.** `WithReplacement` is the v1 default.
2. **Reconstruction analysis** for proximity radius `δ`.
3. **Concrete parameter table** at `ρ ∈ {1/2, 1/4, 1/8, 1/16}`, `λ ∈ {100, 128}`. Headline at the Johnson regime; capacity-regime gain in a single footnote row.
4. **Repairability check** (HSW Def. 22). Likely unavailable in v1, same as FRIDA.

Out of v1 (motivating but deferred): multi-blob batching via `ŵ′`, 2D / kD WHIR-DAS via line-probe `ŵ = Z · eq(ẑ_axis, X)`, and a PoP / aggregation layer for GB-scale via WHIR-flavoured FRIVail variants.

## Implementation scope

The reference implementation has to:

1. **Encode** a blob end-to-end: produce a commitment, a well-formedness (LDT) proof, the extended codeword, and one opening per coset of the L-evaluation table.
2. **Sample** light-client–side: pick `Q` random coset indices, fetch each `(symbol, opening)` pair, and verify them against a once-cached well-formedness proof — the two-phase split that gives DAS its O(log² n) per-sample cost.
3. **Reconstruct**: given any `T` or more validated sample transcripts, run Reed–Solomon decoding on `L` and recover the original data.
4. **Compose end-to-end** in a single-blob, 1D, with-replacement-sampler, systematic-evaluation-encoding configuration, with parameters set at the Johnson radius. Multi-blob batching, alternative samplers, 2D layouts, and message-bound (repairable) openings are explicitly out of v1.

### Fork of `whir-p3` — minimal additive surface

The fork is taken at a pinned commit and is **not rebased during the lifetime of v1**. The guiding constraint on every fork-side change is **fully additive**: every change adds a parallel entry point alongside the existing `MultilinearPcs::{commit, open, verify}` triple, leaves `WhirProof` serialisation untouched, and does not reshuffle Fiat–Shamir ordering.

The minimal v1 surface is two changes:

1. **LDT / per-sample factoring** (`prove_ldt` + `open_sample` + their verifier counterparts). Today, `WhirProver::prove` issues the LDT and the per-position folding-consistency chains in one monolithic transcript; for DAS, the LDT is issued once per blob and gossiped, while the per-position chain is repeated `Q` times by every sampler. The fork extracts the LDT loop and exposes an LDT-only verifier that re-derives folding randomness without re-checking the IOPP queries. This is the standalone coset-opening primitive (Def-8 `Open` / `Ver`) and the two-phase verifier path that gives DAS its O(log² n) per-sample cost.
2. **Documented multi-eval-point and LDT-mode contracts.** Promote two existing-but-incidental capabilities to documented public API: multi-eval-point support and pure-LDT mode (empty `opening_points[0]` plus the Johnson-regime `SecurityAssumption` variant).

### `whir-cc`

Thin Rust wrapper that packages forked `whir-p3` as the four-function ECC interface of HSW Def. 8. Mostly **glue + proof obligation**:

- **Glue.** An `ErasureCodeCommitment` trait, plus a `WhirCc<F, EF, M>` impl that maps `commit` → `whir-p3::commit` + `prove_ldt`; `open` → `open_sample`; `verify` → `verify_sample_path` with a symbol-equality check. The commitment ships in **split** shape — a small (~32-byte) Merkle-root commitment + a one-shot `WellFormedness` LDT proof consumed once via `prepare_verification` to populate a cached `VerifiedCommitment` handle.
- **Proof obligation.** The central theorem of the project — that `WhirCc` is code-binding (Def. 10) at the Johnson radius — is discharged here.

Not in `whir-cc`: the DAS primitive itself, index samplers, Reed–Solomon decoding, the systematic-encoding convention, the 2D-layout helper, the recursive verifier.

### `whir-das`

Realises HSW Def. 1 `(Setup, Encode, V1, V2, Ext)` by instantiating `DAS[CC, Sample]` with `whir-cc` and a pluggable index sampler. Pins three DAS-layer decisions:

1. **Systematic-evaluation encoding.** `data[j] = codeword[j]` for `j < 2^m`, so HSW Lemma 11 (free `L = 1` access) applies without modification. A coefficient-encoding mode is feature-gated for callers interoperating with WHIR-as-a-PCS but is not the default.
2. **Index/coset mapping.** WHIR commits at coset granularity (cosets of size `2^k`); the DAS index space `[N]` is the coset index space. Query budget `Q` is in cosets; reconstruction threshold `T` is in cosets.
3. **Sampler pluggability.** An `IndexSampler` trait with a `quality(Δ, N, Q, ℓ)` method returning the HSW Lemma-3 / 5 / 6 bound. Only `WithReplacement` ships in v1.

The DAS-soundness and DAS-consistency reductions are inherited free from HSW §6.3.

## Repo layout

```
whir-das/
├── crates/
│   ├── whir-cc/        # Erasure-code commitment from WHIR
│   └── whir-das/       # DAS primitive on top of whir-cc
├── Cargo.toml          # workspace
└── README.md
```

## License

MIT. See [LICENSE](LICENSE).
