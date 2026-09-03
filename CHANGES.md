# crossbind — v1001 ↔ v2168 fixes

Same four files as before, under `crates/bedrock-protocol/src/`. Round 2 adds
the changes below on top of round 1.

- `packet_ids.rs`
- `steps/item_stack_v2168.rs`
- `steps/v1001_v2168.rs`
- `steps/crafting_data_v2168.rs`

---

## Round 2 — what the trace log showed

### Confirmed working from round 1

`InventorySlot` (packet 50) logs as `rewrite 12 B -> 12 B`. That is exactly
`inventory_id(1) + slot(1) + no container name(1) + no storage item(1) +
air item(8)`. The old uvarint read would not have landed on 12; the
`ContainerID` fix is parsing the packet exactly.

`ItemStackRequest` (147) now logs `rewrite 75 B -> 44 B` and gets a real
`ItemStackResponse` (148) back at `15 B -> 19 B`. A 15-byte pre-2168 response
carries container data, so the status is OK — the request reached the server
and succeeded instead of being silently blocked. The 75→44 shrink is the
expected shape change: an `ITEM_NAME` string collapsing back to an `int16` id.

`InventoryContent` (49) expanding (`10 B -> 52 B`, `6 B -> 20 B`,
`59 B -> 444 B`) is correct, not a bug. Packet 49 switches descriptor type at
exactly protocol 1001: below it items are `NetworkItemStackDescriptor`, where
air is a single zero byte; at and above it they are
`SerializedNetworkItemStackDescriptor`, where air is the full 8 bytes. Every
one of those three sizes matches that conversion to the byte.

### The remaining drop

```
CraftingData: 36 of 3590 recipes dropped (ingredient not in item registry)
```

That message was a catch-all. `Unrepresentable` was a unit struct, so every
one of a dozen different bail-outs printed the same "not in item registry"
text regardless of the real cause. Two of those bail-outs were avoidable.

### Fix 5 — `COMPLEX_ALIAS` no longer drops the recipe

`ItemDescriptor` has six internal types before 2168; v2168 collapsed the union
into the impl's `toMap()`, which is keyed by case and has no complex-alias
case. The old code treated that as unrepresentable and dropped the whole
recipe.

But a complex alias carries nothing except a name — the same payload
`DEFERRED` carries, and `DEFERRED` was already being sent across as a `"name"`
descriptor. So the alias now goes across as a name too and the receiver
resolves it, in both `crafting_data_v2168.rs` (the recipe list) and
`item_stack_v2168.rs` (`CraftRecipeAuto`, where refusing cancelled the
player's entire craft).

This is lossy in one direction — coming back down the alias re-encodes as
`DEFAULT` or `DEFERRED`, not `COMPLEX_ALIAS`. That only affects a 2168 server
serving a 1001 client, and losing the alias tag beats losing the recipe.

I want to be straight about the confidence here: the reference schema
documents the pre-2168 layout precisely (`crafting.py` lines 69–101, which the
decoder already matched field for field) but does not enumerate the v2168 map
keys. That a complex alias travels under `"name"` is inference from it having
no other payload, not something the schema states. The downside if it's wrong
is bounded — a recipe the client can't resolve, which is what dropping it
produced anyway — and the framing stays well-formed either way.

### Fix 6 — unresolvable names use `DEFERRED` instead of dropping

Going down, an ingredient we had only a name for was dropped when the name
wasn't in the registry. `DEFERRED` is the descriptor that exists precisely to
carry a name the sender couldn't resolve, so it's used now.

### Fix 7 — the notice says what actually happened

`Unrepresentable` carries a reason, and the notice reports the breakdown:

```
CraftingData: 36 of 3590 recipes dropped; ingredient ids missing from the
2121-entry item registry: 704, 705, ...
```

Samples are deduped and capped at 12 ids. If a drop was a width problem rather
than a lookup miss, it says that separately and names the field.

### Fix 8 — a misparsed item registry can no longer stay silent

`cache_item_registry` never checked that the declared entry count landed on
the end of the packet. A wrong entry layout would decode nonsense ids and the
only symptom would be recipes dropping for "missing" ingredients much later.
It now emits a notice with the leftover byte count.

I checked the layout against `ItemData` in the reference (`item.py` lines
15–20: `name`, `int16 id`, `bool`, `ItemVersion`, `CompoundTag`) and it matches
the parser field for field, so I expect this check to stay quiet — it's there
so that if it ever does fire, it says so instead of surfacing as a recipe
problem.

---

## Round 1 (unchanged)

### 1. `steps/item_stack_v2168.rs` — `CraftRecipeAuto` (action id 13)

Was returning `Blocked` for every `CraftRecipeAuto`, so shift-click and
recipe-book crafting silently discarded every request. Now converts both ways
between the pre-2168 `SerializedRecipeIngredient` form (type-tagged
`ItemDescriptor` + `varint32` stack) and the v2168 `RecipeIngredientData` form
(`ItemDescriptorType` variant + `uint16 LE` stack):

| pre-2168 `InternalType`  | v2168 `ItemDescriptorType`  |
| ------------------------ | --------------------------- |
| `INVALID(0)`             | `EMPTY(0)`                  |
| `DEFAULT(1)`, `id != 0`  | `ITEM_NAME(1)` via registry |
| `DEFAULT(1)`, `id == 0`  | `EMPTY(0)` (air canonical)  |
| `MOLANG(2)`              | `MOLANG(2)`                 |
| `ITEM_TAG(3)`            | `ITEM_TAG(3)`               |
| `DEFERRED(4)`            | `ITEM_NAME(1)`              |
| `COMPLEX_ALIAS(5)`       | `ITEM_NAME(1)` (fix 5)      |

The redundant `num_ingredients: uint8` byte between `num_crafts` and the list
is dropped on read and re-emitted from the list length on write.

### 2. `steps/v1001_v2168.rs` — `InventorySlot` stream desync

`inventory_id` was read as `UVarInt`; it is a `ContainerID`, which is `int8`.
When the server sent `NONE` (-1 / `0xFF`) the varint reader saw the high bit
set, ate a second byte, and shifted every field after it. Now read as `Byte`.

Not applied to `inventory_content` (packet 49): that one really does use
`uvarint32` for its container id per the schema. Only packet 50 was wrong.

### 3. `steps/crafting_data_v2168.rs` — molang ingredient offset

The byte after a `"molang"` descriptor's expression was treated as
`int16 molang_version`. Every v2168 `SerializedRecipeIngredient` carries
`aux_value: varint32` in that slot. For the usual `aux=0` this consumed 2
bytes where the wire had 1, shifting `stack_size` and everything after it.

### 4. `packet_ids.rs` — `MAX_PACKET_ID` and 348/349/350

Raised to 350 and added the three ids plus `label()` arms, so they trace by
name rather than as `Packet#348`. No handlers; they pass through.

---

## Still not fixed

- `sub_chunk_shape()` and the `full_chunk_data` LIMITLESS/LIMITED sentinels in
  `steps/v1001_v2168.rs`. The report called all eight candidate shapes
  speculation and this log doesn't disambiguate them. Nothing in the trace
  points at chunk trouble in this session, so I left it alone.
- `ItemData is_component_based` / `item_version` semantics.

## Verification

`cargo test` still has not been run — the container has no Rust toolchain and
no network. Every edit was traced by hand against the reference schema. The
mechanical part of fix 7 (propagating the reason through nine bail-out sites)
was done by script with asserted replacement counts rather than by hand.

Please run `cargo test -p bedrock-protocol` before shipping, and send the new
`CraftingData:` line from the next session — it will now name the ids, which
tells us whether anything is left after fixes 5 and 6.

---

# Round 4 — instrumentation only, no behaviour change

New information this round: the loss is **not** all netherite. Confirmed
missing are the ingot, sword, pickaxe, shovel, hoe and the four armour
pieces, plus `warped_door` and `warped_sign` from the client's recipe log.
`netherite_axe` is **not** missing and its smithing recipe does **not** error,
even though the other eight in that same alphabetical run do.

That asymmetry rules crossbind out as the thing doing the selecting. Nothing
in this codebase knows one item from another: there is no item name table, no
blacklist, and the three packets involved are all item-agnostic.

- `cache_item_registry` forwards packet 162 byte for byte (`write_bytes(&body)`
  before it parses anything), so the client sees exactly the registry BDS sent.
- `creative_content` (145) rewrites only the group category width; it never
  looks at `item_ids` / `item_names` and never skips an entry.
- Recipe *results* are copied through numerically. `read_output` /
  `write_output` move the air early-out in or out and touch nothing else — the
  id is never remapped, so a result cannot be emptied here.

So the two additions below don't fix anything. They make the packet stream
answer the one question three rounds of layout work could not.

### 5. `steps/item_stack_v2168.rs` — registry probe

`cache_item_registry` now reports, per entry it was asked about:

```
item probe: minecraft:netherite_axe present id=609 component_based=false item_version=LEGACY component_nbt=0 B
item probe: 11 of 16 probed names are absent from the registry the server sent: minecraft:netherite_ingot, ...
```

The default list is the eleven names the client complained about plus five
controls (`netherite_axe`, `crimson_door`, `crimson_sign`, `netherite_block`,
`netherite_scrap`). Override with `CROSSBIND_ITEM_PROBE=a,b,c`; disable with
`CROSSBIND_ITEM_PROBE=off`. The summary line also gained a component-based
count and an `item_version` histogram.

### 6. `steps/crafting_data_v2168.rs` — result audit

Every recipe's result is now checked against the cached registry as it
arrives from the server, before re-encoding, and reported by the same recipe
id the client prints:

```
CraftingData: 12 of 3590 recipe results are unusable as they arrive from the
server (results are passed through untouched, so this is upstream of
crossbind); 12 arrive empty (id 0 or stack size 0):
minecraft:netherite_ingot, minecraft:smithing_netherite_sword, ...
```

Silent when the registry hasn't been cached yet, so it can't blame the whole
recipe book on a missing table. Two regression tests cover both arms.

## Reading the next log

- **Probe says the names are absent** → the 1.26.20 server's registry does not
  contain them. Nothing this translator does can conjure an item, and the
  creative-menu symptom follows from the same fact. Look at the server.
- **Probe finds them, audit lists the same recipes the client does** → the
  server is emitting empty results for items it does have. Also upstream.
- **Probe finds them and the audit is silent** → crossbind hands the client a
  valid, in-registry id and the client still calls it empty. Then compare the
  probe lines for `netherite_sword` against `netherite_axe`: if they differ in
  `component_based` or `item_version`, the 1.26.40 client cannot build those
  definitions from a 1.26.20 registry, and the fix is re-emitting those
  entries in the target version's form — a much larger job than field
  shuffling.

`cargo test` still has not been run here (no toolchain, no network).

---

# Round 5 — the probe came back clean, so the dump goes one level down

What the round-4 instruments said:

```
item registry cached: 2122 entries (200 component-based; item_version legacy=48 data_driven=157 none=1917)
item probe: minecraft:netherite_sword present id=774 component_based=false item_version=NONE component_nbt=3 B
item probe: minecraft:netherite_axe   present id=777 component_based=false item_version=NONE component_nbt=3 B
```

All sixteen probed names are present, and **the entry for a name the client
refuses is byte-identical in shape to the entry for the one it accepts** —
same flags, same item_version, same component payload size, adjacent ids. The
"1.26.40 can't build a 1.26.20 data-driven definition" theory from the first
report is dead.

There is also **no `CraftingData: ... recipe results are unusable` line**, so
every one of the 3590 recipes arrived with a non-zero result id, a non-zero
stack size, and an id the 2122-entry registry lists.

Two more things the session establishes:

- The client did not desync. It parsed the whole 823358-byte packet, played
  for six seconds and disconnected cleanly. So the framing is right and the
  failure is **per-recipe**, not a stream offset — exactly twelve recipes out
  of 3590 come out with an empty result.
- The ids make the split look arbitrary from the registry's side:
  774 sword, 775 shovel, 776 pickaxe, **777 axe**, 778 hoe, 779 ingot,
  780–783 armour, 785 crimson_sign, 786 warped_sign, **787 crimson_door**,
  788 warped_door. Everything in that block fails except 777, 785 and 787.

### 7. `steps/crafting_data_v2168.rs` — recipe dump

`CROSSBIND_RECIPE_DUMP=1` dumps the decoded fields and the outgoing bytes for
four recipes: two the client refuses and their near-twins that it accepts.

```
recipe dump minecraft:smithing_netherite_sword [smithing_transform] block=smithing_table
    template id 1234 meta 0 x1
    base     id 552 meta 0 x1
    addition id 779 meta 0 x1
    out[0] id 774 x1 aux 0 block_runtime_id 0 nbt 0 B
    wire 61 B: 25 6d 69 6e 65 ...
```

`smithing_netherite_sword` and `smithing_netherite_axe` differ by one
ingredient and one result id, so their encodings should differ by almost
nothing. Same for `warped_door` against `crimson_door`. Whatever else differs
is the answer. A name in the list that the server never sent is reported as
such — if `smithing_netherite_axe` simply isn't in the packet, that alone
explains why it never errors.

Off unless the variable is set; a comma-separated value dumps exactly those
ids instead of the built-in four.

### 8. `steps/item_stack_v2168.rs` — registry collision check

`cache_item_registry` now reports two names claiming one id, or one name
listed under two ids. That is the only remaining mechanism by which an entry
can be *present* in the packet and still have no item behind it on the far
side — whichever side of the pair loses the insert is simply gone, and every
recipe whose result is that item comes out empty. The server runs plugins
that register items (the registry grew 2121 → 2122 between sessions), so this
is worth ruling in or out. Expected to stay silent.

## Reading the next log

- `smithing_netherite_sword` and `smithing_netherite_axe` dump identically
  except for the name and two ids → crossbind's output is correct and the
  problem is in what the client does with a correct packet.
- They differ structurally → that difference is the bug, and it is here.
- `smithing_netherite_axe: the server never sent a recipe with this id` →
  the control was never a control; the real failing set is "all nine", and
  the question becomes why BDS omits that one recipe.
- `item registry: N name/id collisions` fires → that is the answer.

`cargo test` still has not been run here (no toolchain, no network). Run it
without `CROSSBIND_RECIPE_DUMP` set — a few tests assert on the exact notice
list.

---

# Round 6 — full registry dump, and the recipe-dump bytes came back clean

The recipe dump from the last session settled the question this file was
built to answer: `smithing_netherite_sword`/`smithing_netherite_axe` and
`warped_door`/`crimson_door` come off this translator byte-identical in
shape — same field order, same widths, same optional flags — differing only
in the name strings and the two numeric ids. Same for the ingredient list
(all six inputs by name, `aux=32767`, `x1`), the result descriptor (id, count
1, aux 0, empty extra data), and the trailer (`smithing_table`, net_id). There
is nothing left in the CraftingData encoding to fix.

Combined with the registry probe (all sixteen names present, sequential ids
774–788, no collisions) and the silent result audit (all 3590 results resolve
against the cached registry), every layer crossbind touches has now checked
out clean. The client is refusing an id — 774 for the sword's result — while
accepting an adjacent one — 777 for the axe's — out of the same registry
packet it received unmodified. That has to be settled inside the client's own
parse of packet 162, which a per-recipe or per-item probe can show a sample
of but not compare exhaustively.

### 9. `steps/item_stack_v2168.rs` — full registry TSV dump

`CROSSBIND_ITEM_DUMP=/path/to/file.tsv` writes every cached entry — not just
the probed sixteen — as `name\tid\tcomponent_based\titem_version\tcomponent_nbt_bytes`,
one row per item, on every `cache_item_registry` call. Off unless the path is
set; writes on every connection when it is, so don't leave it on in
production.

The point is a diff, not a read: run this server once and a native 1.26.40
BDS build the client is happy with once, `diff` the two TSVs. Anything that
doesn't line up — an id the target build doesn't use, a version/component
mismatch for the same name — is the drift a handful of individual probe
lines can't surface at once. If the two files are identical for these
sixteen items, the registry layer is fully cleared and the fault is
somewhere client-local (a resource pack, a cached definition from a previous
session, or the client's own id-space assumptions for 1.26.20-vintage data)
rather than anything on the wire.

Two regression tests: dumping is off with the variable unset, and a small
two-entry registry round-trips through the file with the right header and
rows.

## What I'd actually check now, in order

1. `/give @s netherite_axe` and `/give @s netherite_sword` on 1.26.40 against
   this server. If the axe comes out as anything other than a netherite axe,
   the id space itself has drifted between what 1.26.20 assigns and what
   1.26.40 expects, independent of crossbind.
2. Same server, a native 1.26.20 client, no crossbind in the path. Confirms
   whether the twelve items are visible/craftable at all outside translation.
3. The TSV diff described above, once you have a native 1.26.40 BDS build to
   compare against.

`cargo test` still has not been run here (no toolchain, no network).

---

# Round 7 — item id remapping

The two registry dumps settled it. Of 1931 item names present on both sides,
586 changed id between 1.26.20 and 1.26.44, across **ten different deltas**
(+157 ×455, +154 ×51, -465 ×29, +134 ×19, and six more). 478 of the server's
ids name a *different real item* on the client side — `774` is
`netherite_sword` to the server and `blue_cushion` to the client — and 108
name nothing at all. No offset undoes that; the table has to be keyed by name.

The mapping is a clean bijection: no two server ids reach one client id, no
client id has two sources, every target fits `i16`, and no target collides
with an item that kept its id. So a straight substitution is safe and
reversible.

### 10. `item_remap.rs` — the table

`ItemRemap::from_tsv` parses what `compare_registries.py --remap` writes and
refuses anything that would corrupt a connection: a client id claimed twice
(would merge two items and could not be reversed), a server id listed twice,
an id outside `i16`, or an attempt to remap id 0, which is air and is used as
an early-out marker by several codecs. Unlisted ids pass through, so the 1345
items that kept their id cost nothing.

The table is loaded at runtime from `CROSSBIND_ITEM_REMAP`, not compiled in —
it is valid for exactly one pair of versions. With the variable unset, nothing
is remapped and crossbind behaves as it did before.

Held in a process-wide `OnceLock`. That is deliberate: the table is a property
of the server build rather than of a connection, and threading it through
every item call site would have touched far more code than the remap itself.

### 11. Where ids are renumbered

- **Registry (162)** — `cache_item_registry` now re-encodes the packet with
  client ids instead of forwarding it verbatim, so the client is told the
  numbering its own build uses. The internal cache still holds **server** ids:
  recipe ingredients arrive numbered in the server's space and are resolved
  against that map, so remapping it would break every ingredient lookup.
- **All item-bearing packets in the v1001↔v2168 step** — a new `map_item<A, B>`
  helper reads an item in one version's shape, renumbers it, and writes it in
  the other's. `creative_item_stack`, `convert_item`, `convert_legacy_item` and
  `item_stack_to_v1001` all route through it, which covers creative content,
  inventory content/slot, equipment, armour, add-item-actor and the
  transaction paths that share those helpers.
- **Recipe results** — `write_output`, the single exit for a recipe result.
  Ingredients need nothing: v2168 carries them by name, and the numeric side
  resolves against the server-id cache.

Remapping only happens in the step adjacent to the client. The v975→v1001 step
is server-internal — both sides are the server's id space — so it is untouched.

### Not covered yet

Item ids embedded in NBT (`user_data`), item-stack request/response payloads,
and any id inside actor metadata are still forwarded unrenumbered. None of
these carry a bare item id in the paths above, but I have not audited them.

## Using it

```
CROSSBIND_ITEM_REMAP=remap.tsv
```

Expect on enable:

```
item id remap active: 586 ids renumbered between the server and the client
item registry: renumbered 586 of 2121 entries into the client's id space
```

The table is version-specific. **Change either the server or the client build
and it must be recaptured** — dump both registries again and re-run
`compare_registries.py`. A stale table is worse than none, because it will
renumber confidently into the wrong slots.

`cargo test` has still not been run here (no toolchain, no network). The 12 new
tests in `item_remap.rs` cover both directions, pass-through, air, round-trip
losslessness, and every rejection case.

---

# Round 8 — the remap has to cover the whole registry

Round 7's table worked: the missing items came back. It also introduced a
golden sword called "近战", which turned out to be the visible tip of **127 id
collisions**.

The table was built from names present in *both* registries. The server's 190
server-only entries — 180 of them from the bedwars addon — were not in it, so
they kept their original ids while vanilla names were renumbered on top of
them. `bedwars:category_melee` sits on 322 and `minecraft:mace` was moved onto
322; `bedwars:category_ranged` sits on 325 and `minecraft:golden_sword` was
moved onto 325. Two entries per slot, one loses the insert, and the survivor
renders under the other one's identity.

A partial renumbering is not a smaller version of a correct one. It is broken
in a way the original problem was not.

### 12. The mapping is now built at runtime, over the whole registry

`item_remap.rs` no longer loads a precomputed diff. It loads the **client's
registry dump** — the file `endstone-itemdump` writes on a native server of the
client's version, handed over unedited — and builds the mapping when the server
sends its own registry:

1. Names the client knows take the client's id. Claimed first, non-negotiable.
2. Server-only names keep their id when nothing claims it.
3. Otherwise they are relocated to a free id, **preserving sign**: a negative id
   is a block item, and a positive replacement would change what it is.

Checked against the real pair: 586 renumbered, 1345 already agreed, 63
server-only kept, 127 relocated, 0 unplaceable, and **0 collisions**. All ids
stay in `i16`, no sign flips, and the mapping is a bijection so the serverbound
direction reverses exactly. `bedwars:brown_boots` moves 258 → 890, above the
client's highest id.

`emit_registry` also counts duplicate ids in what it is about to send and says
so. That check would have caught round 7 on the first connection instead of via
a golden sword.

## Configuration changed

`CROSSBIND_ITEM_REMAP` (a diff) is replaced by `CROSSBIND_CLIENT_ITEMS` (the
client's registry dump):

```
CROSSBIND_CLIENT_ITEMS=item_registry_1_26_44.tsv
```

`remap.tsv` is no longer used and can be deleted. Expect:

```
client item table loaded: 1976 entries; the mapping is built when the server sends its registry
item registry: renumbered 586 of 2121 entries into the client's id space
  (586 renumbered, 1345 already agreed, 63 server-only kept, 127 relocated)
```

Still version-specific: recapture the client dump whenever either side moves.
But the addon no longer needs its own handling — its items are placed from
whatever the server actually registers.

`cargo test` has still not been run here (no toolchain). 14 tests in
`item_remap.rs` cover both directions, relocation, sign preservation,
bijectivity, air, and every parse rejection.

---

# Round 9 — the table belongs in the build, and the log belongs to the operator

Two things round 8 got wrong.

**The table was configuration.** `CROSSBIND_CLIENT_ITEMS` made a correctness-
critical file something an operator had to place and point at. A missing path
meant silently forwarding wrong ids; a stale one meant confidently renumbering
into the wrong slots. Neither failure is an operator's to make.

`data/item_registry_v2168.tsv` now lives in the crate and is pulled in with
`include_str!`. There is nothing to configure, nothing to ship alongside the
binary, and a broken table is a build failure rather than a runtime surprise —
a test parses the embedded copy and asserts the anchors
(`netherite_sword` 617, `netherite_axe` 620, `warped_door` 631) still hold, so
a table that stops matching the target client fails `cargo test`.

`CROSSBIND_CLIENT_ITEMS` and `CROSSBIND_ITEM_REMAP` are both gone.

**The log was written for me, not for you.** Sixteen probe lines, a registry
histogram, and a mapping summary on *every player join* is debugging output
sitting in a production server's log.

New module `diag.rs`, one switch, `CROSSBIND_DIAG=1`. Behind it: the item
probes, the registry histogram, the mapping breakdown, the per-collision
detail, `describe_support` and the set-score layout. `CROSSBIND_RECIPE_DUMP`
now also requires it.

Unconditional output is now only:

```
server speaks 1.26.20 (protocol 975)
item id translation ready (1976 client items)
client connected as 1.26.44 (protocol 2168), server speaks 1.26.20 (protocol 975)
item ids: 713 of 2121 renumbered for the client
```

and problems, prefixed `WARNING:` and worded for someone who has to act on
them rather than someone reading the source. The mapping summary is printed
once per server run rather than once per join — `build_once` now reports
whether the call is what built it.

`cargo test` has still not been run here (no toolchain).
