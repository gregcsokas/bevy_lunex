# Session Report: Flow Layout — Wrapping, Grid & Showcase

**Scope of this report:** the second half of the flow layout rework (Phases 2–3 of the
Sp-margin architecture), plus two follow-up fixes to the showcase example.
The first half (Sp unit, margins, `align`/`justify`, inverted directions, engine rework,
51 tests) was completed and verified in a prior session; this report covers everything
built on top of it.

---

## 1. What was built

### 1.1 API (`crate/src/layouts.rs`)

New fields on `UiLayoutTypeFlow`, all with builders:

| Field | Builder | Meaning |
|---|---|---|
| `wrap: bool` | `.wrapping()` | Pack children onto multiple lines along the main axis (greedy, like `flex-wrap: wrap`). Named `wrapping()` because `wrap()` collided with an existing conversion method. |
| `flipped: bool` | `.flipped()` | Stack lines from the opposite edge: the *first* line sits at the cross-axis end, later lines wrap toward the start. Placement-time only; no effect on a single line. |
| `grid: Vec<UiFlowSize>` | `.grid(impl Into<Vec<UiFlowSize>>)` | Explicit main-axis tracks (`Fit` / `Fixed` / `Sp` / `Grow`). Implicitly enables wrap mode. |
| `grid_wrap: bool` (default `true`) | `.grid_wrap(bool)` | If true, full lines wrap onto the next line. If false, everything stays on one line; items beyond the defined tracks flow into implicit `Fit` tracks. |

**Breaking change:** `UiLayoutTypeFlow` (and therefore `UiLayoutType`) is now `Clone`
but no longer `Copy` — forced by `grid: Vec<_>`. All internal `.config` reads in the
engine were converted to `.config.clone()`. Several existing unit tests needed
`.clone()` added where they reused configs across blends.

**Blending:** `lerp_flow_config` handles the new fields by picking the dominant state
(`t >= 0.5`), consistent with `direction`/`justify`: `wrap`, `flipped`, `grid`,
`grid_wrap` all snap; margins/padding/gap/align still interpolate numerically.

### 1.2 Engine (`crate/src/flow.rs`)

The flow engine was restructured around a **line-based** model. A single-line
container is now just the degenerate case of one line — no separate code path.

**New helpers:**

- `is_wrap_mode(config)` — `config.wrap || !config.grid.is_empty()` (grid implies wrap).
- `pack_lines_greedy(footprints, gap, inner_main)` — greedy line packing by main-axis
  footprints (fixed margins + size). An item wider than a whole line gets a line of its
  own. Items whose base size is 0 (pure `Sp` sizing) pack onto one line — flexbox-like.

**`compute()` — bounded fixpoint.** When any item in the subtree is in wrap mode, the
whole pipeline (bottom-up content → root resolution → top-down sizing → placement)
runs up to 3 iterations. A `wrap_dirty` flag on `FlowLayout` is set by `process()`
whenever a wrapping container's cross size changed by more than ε in the last top-down
pass; the loop exits early when clean. Rationale: a wrapping container derives its
cross size from its lines, which are only known top-down — ancestors need a re-run to
hug the correct size.

**`compute_content()` — wrap-aware bottom-up hugs.**

- Greedy-wrap containers estimate their line packing when the main extent is knowable
  bottom-up: an intrinsically resolvable `Fixed` main size, or the size carried over
  from the previous fixpoint iteration (read before the function's `size = content`
  clobber). The cross hug then becomes Σ (per-line max footprint) + gaps + padding.
- **Wrap containers' main-axis minimum is the largest single child footprint**, not the
  single-line sum: they can always compress by wrapping. This is also what lets the
  water-level overflow shrink push a Grow-main wrap container down to a width where
  wrapping actually kicks in (previously the single-line min floored it and the
  overflow persisted forever).
- **Grid line structure is extent-independent** (chunking by track count), so grid
  containers compute their line groups and track bases bottom-up (`Fit` = footprint,
  `Fixed` = explicit, `Grow`/`Sp` = margins only) without needing any estimate. A
  `Fit` main-sized grid container therefore hugs its longest line directly, with no
  fixpoint oscillation.

**`process()` — unified line-based top-down pass.**

1. `Fixed` children resolution (unchanged).
2. Main-axis margins split into `MarginPart { fixed, weight }` and main footprints.
3. **Line packing**: grid → chunks of `grid.len()` (or one line when `!grid_wrap`);
   wrap → `pack_lines_greedy`; otherwise a single line.
4. **Per-line main distribution**:
   - Non-grid lines: existing `distribute_leftover` (shared `Sp` pool with max-pin
     re-normalization) or per-line water-level shrink on overflow. The `Sp` pool is
     **per line** — claims never span lines.
   - Grid lines: new `distribute_grid_tracks` — `Fit` tracks hug their item's footprint,
     `Fixed` tracks are explicit, `Sp`/`Grow` tracks claim shares of the line's leftover
     *alongside* the item's own `Sp` margins (one shared pool). Items are stretched to
     their track minus their fixed margins, floored at their minimum. Grid intentionally
     has no overflow shrink.
5. **Line cross extents**: wrap mode → per-line max footprint; single line → the
   parent's inner cross axis (exact legacy behavior).
6. **Wrapping container cross size override**: a `Fit` cross-sized wrap container is
   resized to Σ line extents + gaps + padding (clamped by its own min/max), `content`
   kept in sync, `wrap_dirty` set when it changed by > ε.
7. **Cross resolution within the line extent** (per child): per-child `Sp` pool,
   classic clamping for resizable children bounded by the *line* extent, and — new —
   wrap-mode children are never clamped *down* by their parent (they own their cross
   size, which comes from their lines); they are only floored at their minimum.
   `Grow`-cross children still fill their line extent.

**`place()` — line-aware placement.** Children place sequentially within each line
(inverted directions mirror main-axis placement as before); lines stack along the cross
axis cumulatively, or — when `flipped` — from the end edge via
`start = pad + inner − Σ_{j≤k} line_cross[j] − k·gap`, i.e. the first line at the cross
end and later lines wrapping toward the start. Child cross position = line start + its
resolved cross margin. No align-content beyond this for v1 (documented).

`FlowItem` gained `lines: Vec<Vec<usize>>` (indices into the container's children) and
`line_cross: Vec<f32>`, set by `process()`, consumed by `place()`.

### 1.3 Tests

All prior tests kept green (single-line path must be behavior-identical — it is,
because a single line's extent *is* the inner cross axis and the per-line pool *is*
the old shared pool).

New unit tests in `flow.rs` (64 total now, was 51):

- `wrap_packs_lines_and_hugs_cross` — greedy packing, cross hug 2 lines, positions.
- `wrap_grow_claims_resolve_per_line` — two claims share line 1's leftover (40 → both
  120), lone item on line 2 claims all 150.
- `wrap_aligns_within_line_extent` — `align: END` pushes a short child to the line bottom.
- `wrap_grow_cross_fills_line_extent` — grow-cross child fills the line extent.
- `wrap_oversized_item_gets_own_line`.
- `wrap_flipped_stacks_lines_from_end` — exact flipped positions.
- `wrap_grow_main_container_settles_through_fixpoint` — the fixpoint test: a
  Grow-width wrap container next to a fixed sibling in a Fit parent; the parent's
  height only becomes correct on the second iteration.
- `grid_fit_tracks_hug_items`, `grid_fixed_tracks_stretch_items`,
  `grid_sp_tracks_share_leftover` (1:3 split → 97.5 / 292.5),
  `grid_grow_tracks_wrap_lines` (2×195 tracks, lone 5th track takes the whole row =
  400), `grid_no_wrap_stays_single_line` (implicit `Fit` tracks beyond the definitions),
  `vertical_grid_tracks_run_along_y` (tracks along the flow direction, wrapped lines
  stack along x).

New integration tests in `crate/tests/flow_layout.rs` (15 total now, was 13):
`flow_wrap_packs_lines` and `flow_grid_tracks_and_lines` through the full ECS path
(observers → `system_layout_compute` → transforms).

`public_api_doc_examples_compile` (the doctest mirror — doctests themselves are
**never run** in this repo, see constraints) extended with `wrapping()`, `flipped()`,
`grid(...)`, `grid_wrap(false)` usage.

### 1.4 Docs

- `docs/src/chapters/flow.md`: new "Line wrapping" and "Grid tracks" sections — greedy
  packing, per-line `Sp` pools, line extents, `Fit` cross hugging + fixpoint, the
  "main sizing must not be `Fit`" requirement for greedy wrap, flipped stacking, grid
  track semantics, `grid_wrap(false)` behavior, vertical grids.
- `docs/src/chapters/2d/layouts.md`: flow field list extended with `wrap` and `grid`.

### 1.5 Showcase example (`examples/flow_showcase/`)

New example demonstrating the whole flow model in three columns:

1. **Ordering** — the four stack directions (LTR / RTL / TTB / BTT) with 4 items each.
2. **Justify** — all six justification modes (Start / Center / End / SpaceBetween /
   SpaceEvenly / SpaceAround), `align: CENTER`, fixed-size items in grow-width rows.
3. **Wrap & Grid** — a `Fit`-height wrapping container (7 items), a fixed-height
   flipped wrapping container (4 items, see §3), and a grid with mixed track types
   (`[Fixed(48), Grow, Sp(1)]`, 7 cells → 3 lines).

Plus **drag-to-resize** (see §3.2).

---

## 2. Key decisions & rationale

- **One shared `Sp` pool per line** (margins + sizing claims together), not separate
  margin/justify pools. Chosen by the user; keeps "everything is Sp margins"
  semantics uniform and lets justify templates interact with grow claims naturally.
- **Lines as first-class entities** with explicit `line_cross` extents, instead of
  nested flow resolution. Simpler, one engine, and the single-line path stays
  bit-identical to the pre-wrap engine (verified by all legacy tests passing).
- **Bounded fixpoint (≤3 iterations, ε-gated)** instead of Clay's dependency-ordered
  re-layout passes. Wrapping cross sizes propagate at most one ancestor level per
  iteration; 3 covers the practical nesting depth (wrap container → Fit parent →
  grandparent) and the early-exit keeps non-wrap trees at exactly one iteration.
- **Wrap containers own their cross size**: excluded from their parent's cross
  down-clamping. Otherwise a `Fit`-cross wrap container would be clamped to a stale
  single-line estimate by its parent before its own `process()` runs, defeating the
  fixpoint. `Grow`-cross children still fill their line extent.
- **Grid lines are extent-independent** (chunking), so grid content hugs are exact
  bottom-up — no estimate, no oscillation. Greedy wrap fundamentally needs the
  top-down main size, hence the estimate/fixpoint machinery only for it.
- **`min` of a wrap container = largest child footprint** (min-content, like CSS).
  This is what allows overflow-shrink to compress a wrap container into an
  actually-wrapping width.
- **Grid has no overflow shrink**: tracks are authoritative; items that can't fit
  overflow (CSS-like, documented). Grid containers with `Fit` cross sizing still hug
  their lines via the same override as wrap.
- **No clipping anywhere**: overflowing content renders past the container
  (CSS flexbox behavior). Containment is a rendering concern, not layout.

## 3. Issues found & fixed during review-by-user

### 3.1 Showcase overflow (user-reported)

At narrow window widths, the *flipped* wrap container (fixed `height(Ab(116.0))`,
7 items of 64px) needed 220px (items wrapped one per line) → lines rendered past the
container, z-fighting with sibling containers. Diagnosis: engine behaved exactly as
configured (fixed height + unshrinkable fixed items + no clipping = CSS-like
overflow); the example's fixed height was a bad choice. **Fix:** 4 items and
`height(Ab(130.0))` = exact worst case (4×24 + 3×6 + 2×8 = 130), so the narrowest
state exactly fills and wider states show the flipped bottom-stacking with visible
space above.

### 3.2 Drag-to-resize (user request)

Resizing the OS window to demo dynamic layout thrashes the WGPU swapchain buffer.
The example now resizes the **UI root inside a fixed window**: hold LMB and drag —
the root spans from the window's top-left corner to the cursor (`cursor_position()`
clamped to `200×100`..window size), with the top-left kept pinned via a shared
`top_left_anchor` helper (the root box is centered on its `Transform`). `UiFetchFromCamera`
was removed from the root; mutating `Dimension` re-triggers `system_layout_compute`
via `Changed<Dimension>`. The root carries a backdrop sprite (auto-sized from
`Dimension` by `system_pipe_sprite_size_from_dimension`) to visualize the region.

Accepted tradeoff (user-approved): below ~660px root width the fixed 64px items in
the Justify rows overlap the right column — fixed sizes never shrink, CSS-like.

## 4. Verification

- `cargo test -p bevy_lunex --lib` — **64 passed** (was 51).
- `cargo test -p bevy_lunex --test flow_layout` — **15 passed** (was 13).
- `--test layout_recompute`, `--test state_change_detection` — 1 + 1 passed.
- `cargo clippy -p bevy_lunex --all-targets` — clean (one targeted
  `#[allow(clippy::too_many_arguments)]` on `distribute_grid_tracks`, internal
  single-call helper).
- `cargo check --workspace`, `cargo clippy -p flow_showcase` — clean.
- **Doctests are NOT run** (standing user directive: each doctest binary links all of
  Bevy, >10 GB RAM thrash). Doc examples are verified via the
  `public_api_doc_examples_compile` mirror unit test instead.
- `cargo fmt` is NOT run (standing directive: repo uses custom manual formatting).

## 5. Files touched

| File | Change |
|---|---|
| `crate/src/flow.rs` | Line-based engine rework (wrap/grid/flipped), fixpoint, `distribute_grid_tracks`, `pack_lines_greedy`, `is_wrap_mode`; 13 new unit tests; doc-example mirror extended. |
| `crate/src/layouts.rs` | `wrap`/`flipped`/`grid`/`grid_wrap` fields + builders; `Copy` dropped; lerp handles new fields. |
| `crate/tests/flow_layout.rs` | 2 new integration tests (wrap, grid). |
| `docs/src/chapters/flow.md` | "Line wrapping" + "Grid tracks" sections. |
| `docs/src/chapters/2d/layouts.md` | Flow field list extended. |
| `examples/flow_showcase/` | New example (ordering, justify, wrap/flipped/grid panels + drag-to-resize). |

---

## 6. Post-review fixes

A follow-up code review of the wrap/grid engine, the ECS integration and the showcase
led to these fixes on top of the work described above:

- **Wrap containers' main-axis minimum now includes their own padding** (previously
  the largest child footprint only, letting overflow-shrink compress a wrap container
  below `padding + largest item`).
- **`.grid(vec![])` no longer silently disables wrapping** set through `.wrapping()`
  (the builder used to force `wrap = false` for an empty track list).
- **Grid items are capped at their own `max` clamps** when stretched to their track,
  consistent with the non-grid leftover distribution.
- `wrap_dirty` is reset at the start of each `compute()` (a subtree that exhausted
  its fixpoint could leak the flag into the next subtree's loop); the bound is now
  the named `FIXPOINT_MAX_ITERATIONS` constant.
- Grid line chunking extracted into `pack_lines_grid` (was duplicated between the
  bottom-up and top-down passes); cross-axis margin evaluation deduplicated in
  `process()`.
- The wrap-text width feedback in `system_layout_compute` is now a single pass over
  all wrap-text items after the traversal (was two duplicated blocks, one of which
  compared descendants' widths against the subtree root's `TextBounds`).
- Showcase: flipped wrap container height corrected to the true worst case (130),
  `top_left_anchor` returns `Vec2`.
- New unit tests: `wrap_minimum_includes_padding`, `empty_grid_keeps_wrapping_enabled`,
  `grid_items_respect_max_clamp`, `grid_flipped_stacks_lines_from_end` (68 total, was 64).
