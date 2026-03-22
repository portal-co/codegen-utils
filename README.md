# codegen-utils

A Cargo workspace of Rust crates providing trait-based abstractions and
implementations for compiler intermediate representations: control flow
graphs (CFG), static single assignment (SSA) form, and code generation
backends.

The crates are designed to be composable via Rust's trait system. Most core
crates are `#![no_std]` compatible (requiring only `alloc`). The workspace
uses Cargo's resolver v2.

---

## Workspace members

The workspace `Cargo.toml` lists 12 crates. Four additional crates exist on
disk (`ast-traits`, `siftify`, `ssa-pliron-compat`, `ssa-trace`) but are
not included in the workspace members list.

---

## Crates

### `cfg-traits` (v0.2.3, CC0-1.0)

Core CFG abstraction layer. Defines three traits that everything else builds
on:

- `Func` — a function containing an arena of `Block`s and an entry block.
- `Block<F>` — holds a `Terminator`.
- `Term<F>` — iterates `Target`s (successors).
- `Target<F>` — a single successor edge; provides `block()`/`block_mut()`.

`Either<A,B>` gets a blanket `Term` impl so you can compose terminator
types without boilerplate. The `util` module adds `FuncViaCfg<T,W>` (a
newtype that pairs an arbitrary CFG `T` with an entry block) and a
`func_via_cfg!` macro to generate the corresponding wrapper struct.

Type aliases `BlockI<F>`, `TermI<F>`, `TargetI<F>` project through the
arena index to the concrete output types.

Dependencies: `arena-traits`, `either`.

---

### `ssa-traits` (v0.2.3, CC0-1.0)

Extends `cfg-traits` with SSA value semantics.

- `Func` — adds a `Value` type, a `Values` arena, and accessors.
- `Block<F>` — adds `insts()` (iterate value IDs in order) and
  `add_inst(func, block, value)`.
- `Value<F>` — marker; requires `HasValues<F>`.
- `TypedFunc` / `TypedBlock` / `TypedValue` — optional layer for typed
  IRs; `TypedFunc::add_blockparam` adds a block parameter with a type.
- `HasValues<F>` / `HasChainableValues<F>` — iterate or mutably iterate
  the `F::Value` IDs held by an operand.
- `Target<F>` / `Term<F>` — SSA-aware versions that also implement
  `HasValues`.
- `Builder<F>` / `CpsBuilder<F>` — helper traits for CPS-style IR
  construction; closures `FnOnce(&mut F, F::Block) -> Result<(R, F::Block)>`
  automatically implement `Builder`.
- `op::OpValue<F,O>` — disassembly/reassembly trait for pattern-matching
  specific op shapes out of a value type, with composable `Either`-based
  chaining.

`Val<F>` and `Vec<F::Value>` both implement `HasValues` and
`HasChainableValues`.

Dependencies: `anyhow` (no-std), `arena-traits`, `cfg-traits`, `either`.

---

### `ssa-impls` (v0.2.3, CC0-1.0)

Concrete algorithms over any IR that satisfies `cfg-traits`/`ssa-traits`.

**`cfg` module** — iterative DFS postorder traversal (`postorder`,
`calculate_postorder`).

**`dom` module** — Cooper/Harvey/Kennedy "Simple, Fast Dominance
Algorithm" (adapted from regalloc.rs, Apache-2.0/LLVM-exception).
Derives from a postorder walk; produces a `BTreeMap<Option<Block>,
Block>` immediate-dominator tree. Public functions: `domtree`,
`dominates`, `calculate`.

**`maxssa` module** — max-SSA conversion pass (`maxssa`). For each block,
identifies values used but not defined there, adds block parameters via
`TypedFunc::add_blockparam`, and rewrites all branch targets to thread
the values through. Works in two passes: collect new args, then rewrite
uses and branch args.

**`reducify` module** — CFG reducibility pass (`Reducifier::run`). Uses
RPO to find back-edges and assign header sets; detects irreducible loops
(edges that enter a loop region without going through the header); when
irreducible edges exist, applies max-SSA then clones basic blocks to
separate execution contexts, rewriting value and block maps accordingly.

`preds(f, k)` — iterator over predecessor blocks of `k`.
`add_phi(f, k, ty, trial)` — adds a block parameter and propagates a
trial value from each predecessor.

Dependencies: `arena-traits`, `cfg-traits`, `ssa-traits`.

---

### `ssa-reloop` (v0.2.3, CC0-1.0)

Bridges the SSA IR to the `relooper` crate. Given a `Func` whose `Block`
type implements `RelooperLabel`, calls `ssa-impls::dom::domtree` to build
the dominator tree, then feeds reachable blocks (those dominated by the
entry) and their successors into `relooper::reloop`, returning a
`Box<ShapedBlock<F::Block>>`.

Dependencies: `arena-traits`, `cfg-traits`, `relooper`, `ssa-impls`,
`ssa-traits`.

---

### `ssa-rust` (v0.2.3, CC0-1.0)

Emits Rust `TokenStream` from an SSA IR using `ssa-reloop` for structured
control flow.

Defines `RsFunc` (composed trait bound), `Rs<F>` (emit to `TokenStream`),
`RsId<F>` (emit as an `Ident`), `RsTerm<F>` (emit a terminator given a
block-to-tokens closure), and `RsOp<F>` (emit an operation given args and
block args).

`go(params, f, entry)` — top-level emitter. Declares `Option<T>`
variables for every SSA value and every block parameter, calls
`ssa-reloop::go` to get a `ShapedBlock`, then recurses through it via
`block()`. Branch control flow uses labeled `loop` / `continue` /
`break` following relooper's `BranchMode`. A `cff` integer variable
handles `Multiple` shaped blocks (the "control-flow flattening" pattern).

`render_target` emits the assignment sequence for a branch target's
arguments plus the goto expression.

With `feature = "ssa-canon"`, provides `Rs` for `ssa_canon::Value` and
`RsTerm` for `ssa_canon::Target`. With `feature = "id-arena"`, provides
`RsId` for `id_arena::Id<T>`.

Dependencies: `anyhow`, `arena-traits`, `cfg-traits`, `either`,
`proc-macro2`, `quasiquote`, `quote`, `relooper`, `ssa-canon` (optional),
`ssa-reloop`, `ssa-traits`, `syn`.

---

### `ssa-cc` (v0.2.3, CC0-1.0)

Emits C source code from an SSA IR.

`CCFunc` — composed trait bound requiring types/blocks/values/terminators
all implement `C<F>` (the `fn c(&self, f: &F) -> Result<String>` trait).

`cc(s, entry)` — generates a C function body: parameter list, variable
declarations (one per block parameter that isn't the entry, plus one per
SSA value), a `goto` to the entry block label, then a sequence of
labeled basic blocks (`BB<n>:`) each containing value assignments and a
terminator.

`render_target` — emits the argument-copy sequence and `goto` for a branch.

With `feature = "id-arena"`: `C` for `id_arena::Id<T>` (formats as
`x<index>`). With `feature = "ssa-canon"`: `C` for `ssa_canon::Value`
and `ssa_canon::Target`. `Either<A,B>` gets blanket `C` and `COp` impls.

Dependencies: `anyhow`, `arena-traits`, `cfg-traits`, `either`,
`id-arena` (optional), `ssa-canon` (optional), `ssa-traits`.

---

### `ssa-canon` (v0.2.3, CC0-1.0)

A concrete, generic SSA IR parameterized by `<O, T, Y>` (operation type,
terminator type, type annotation):

- `Value<O,T,Y>` — either `Op(O, args, block_args, ty)` or
  `Param(index, block, ty)`.
- `Block<O,T,Y>` — holds `term: T`, `insts: Vec<Id<Value>>`, and
  `params: Vec<(Y, Id<Value>)>`.
- `Target<O,T,Y>` — `{ args: Vec<Id<Value>>, block: Id<Block> }`.
- `Func<O,T,Y>` — `{ vals: Arena<Value>, blocks: Arena<Block>, entry }`.

Implements `cfg_traits::Func`, `ssa_traits::Func`, `ssa_traits::TypedFunc`,
`ssa_traits::Block`, `ssa_traits::TypedBlock`, `ssa_traits::Value`,
`ssa_traits::TypedValue`, `ssa_traits::Target`, and all the corresponding
`cfg_traits` counterparts.

Also implements `OpValue<Func<O,T,Y>, CanonOp<X>>` for `Value<O,T,Y>`
when `O: Sift<X>`, enabling pattern-matching ops out of the value via
the `sift-trait` mechanism. Uses `unsafe { mem::transmute }` to reinterpret
arena IDs across the type change.

Backed by `id-arena`. Uses `arena-traits` with the `id-arena` feature.

Dependencies: `anyhow`, `arena-traits` (id-arena feature), `cfg-traits`,
`id-arena`, `sift-trait`, `ssa-traits`.

---

### `ssa-translation` (v0.2.3, CC0-1.0)

Generic SSA-to-SSA translation framework.

`Translator<F: TypedFunc, G: Func>` — the core translation trait with
associated types `Meta` (how a source value maps to target values) and
`Instance` (per-block state). Methods: `add_blockparam`, `emit_val`,
`emit_term`.

`State<F,G,T>` — memoizes already-translated `(source_block, instance)`
pairs in a `BTreeMap`; `State::go` drives the translation loop, calling
`add_blockparam` for each typed block param and `emit_val` for each
instruction before `emit_term`.

`CarryTranslator<F,G>` — a blanket refinement for translators where
`Meta: ValSer<G::Value>` and `Instance: EqIter<Item=(F::Ty, Meta::Kind)>`.

`ai` module — `AI<H>` wrapper that implements `Translator` given a
`Handler<F,G>` that knows how to `stamp`/`unstamp` kind-annotated values
(`AnyKind` from the `valser` crate) and emit individual values/terminators.
`emit_target` reconstructs a target in the output IR by gathering values,
inferring types via `ty()`, and calling `go` for the target block.

Dependencies: `anyhow`, `arena-traits`, `cfg-traits`, `either`,
`ssa-traits`, `valser`.

---

### `sift-trait` (v0.2.3, CC0-1.0)

Single trait: `Sift<T>` — extracts a `T` from `Self` or returns a
`Residue`. Methods: `sift(self) -> Result<T, Residue>`, `of(T) -> Self`,
`lift(Residue) -> Self`.

Blanket impl for `Either<T,U>`: if `A: Sift<T>` and `A::Residue: Sift<U>`,
then `A: Sift<Either<T,U>>`, with `Residue = <<A as Sift<T>>::Residue as
Sift<U>>::Residue`. This chains sifting left-first, falling through to
right.

Used by `ssa-canon` to pattern-match op variants and by `tac-traits` to
distinguish register LValues from other LValues.

Dependencies: `either`.

---

### `register-machine-traits` (v0.2.3, CC0-1.0)

Extends `cfg-traits::Func` with a register file:

```rust
pub trait Func: cfg_traits::Func {
    type Reg;
    type Regs: IndexIter<Self::Reg>;
    fn regs(&self) -> &Self::Regs;
    fn regs_mut(&mut self) -> &mut Self::Regs;
}
```

`#![no_std]`. No algorithms; purely a trait definition.

Dependencies: `arena-traits`, `cfg-traits`.

---

### `tac-traits` (v0.2.3, CC0-1.0)

Three-address code layer on top of `register-machine-traits`.

`Func` — blanket impl over any `register_machine_traits::Func` whose
blocks arena yields `Block<Self>`.

`Block<F>` — extends `cfg_traits::Block` with:
- `type Item` — the instruction payload.
- `type LValue: Sift<F::Reg, Residue: AsRef<F::Reg> + AsMut<F::Reg>>` —
  either exactly a register, or a residue that can borrow as a register.
  Also requires `AsRef<F::Reg> + AsMut<F::Reg>`.
- `values()` / `values_mut()` — iterate `(&LValue, &Item)` pairs.
- `add_value(lvalue, item)` — append an instruction.

Helper type aliases: `ItemI<F>`, `LValueI<F>`, `LValueNonRegI<F>`.

`#![no_std]`.

Dependencies: `arena-traits`, `cfg-traits`, `register-machine-traits`,
`sift-trait`.

---

### `onion` (published as `type-onion`, v0.2.3, CC0-1.0)

Utility for merging multiple lists of items by equality, tracking index
mappings.

`union(iterators)` — takes an iterator of iterators, deduplicates items
by `==` into a single `vals: Vec<A>`, and records for each input list
the positions (`poss: Vec<Vec<usize>>`) of its items in the merged
`vals`.

`Union::create(i, default, vals)` — given a per-input-list index `i` and
a fresh iterator of values, constructs a full-length `Vec<B>` by writing
defaults for items not in list `i` and the provided values for items that
are.

`#![no_std]`.

Dependencies: none.

---

## Crates on disk but not in the workspace

### `ssa-trace`

Defines a `Trace<F,G>` trait for abstract interpretation / symbolic
execution: `run` a block producing a `State: ValSer<G::Value>`, and
`transfer` across an edge producing a new `Instance`. The `Tracer<F,G,T>`
struct wraps the implementation and memoizes `(block, instance) ->
G::Block` results.

Dependencies: `anyhow`, `arena-traits`, `cfg-traits`, `ssa-traits`,
`valser`.

### `ssa-pliron-compat`

Bridge between the `pliron` MLIR-style IR framework and `ssa-traits`.

`PlironCompatOp<F: ssa_traits::Func>` — implemented by `pliron::op::Op`
subtypes; `to_ssa_traits` translates the op into the target SSA IR given
value/block maps.

`pliron_compat_op!(F => TraitName)` — macro that generates a blanket alias
trait and registers a `linkme` distributed-slice entry for pliron's op
interface dependency system.

Dependencies: `anyhow`, `arena-traits`, `cfg-traits`, `linkme`, `pliron`,
`ssa-traits`.

### `ast-traits`

Minimal AST abstraction. Defines `Ast` with associated types
`Value<Sub>` and `Control<Sub>`, and `AstImpl<A>` as an enum of `Op` or
`Control`. No dependencies.

### `siftify`

A `proc-macro2`/`quote`/`syn`-based code generation helper (`patch`) that
rewrites enum definitions: adds a generic type parameter
`__Pattern_<Variant>` (defaulting to `()`) for each variant, and injects
a corresponding field into each variant. Not registered in the workspace.

Dependencies: `proc-macro2`, `quote`, `syn`.

---

## Dependency relationships (workspace crates only)

```
cfg-traits
  └── ssa-traits
        └── ssa-impls
              └── ssa-reloop ──── ssa-rust
sift-trait ───── ssa-canon ─────── ssa-cc
register-machine-traits
  └── tac-traits
onion (standalone)
```

The external crates used across the workspace are: `arena-traits`,
`either`, `anyhow`, `id-arena`, `relooper`, `valser`, `proc-macro2`,
`quote`, `quasiquote`, `syn`.

---

## Versioning

All workspace-member crates are at version `0.2.3` (current) with tags
for each crate published back to `v0.1.x`. Release branches visible in
remotes: `v0.2`, `v0.4`, `dev/0.3`. The workspace `Cargo.toml` declares
workspace-level `id-arena = "2.2.1"` used by `ssa-canon` and optionally
by `ssa-cc` and `ssa-rust`.
