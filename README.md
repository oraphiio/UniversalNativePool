# UniversalNativePool

**A high-performance, designer-driven object pooling GDExtension for Godot 4.7 / 4.8 — built with godot-rust (gdext) 0.5.5, Rust edition 2024.**

UniversalNativePool replaces per-object instantiation cost with a *channel-based dormant matrix*: scenes are pre-allocated or chunk-allocated once, recycled forever, and level layouts authored with editor placeholders are harvested at startup and restorable at any time as checkpoint resets.

---

## ✨ Features

- **Type-agnostic pooling** — the pool and its channels are plain `Node`s, so any `PackedScene` can be pooled: `Node3D`, `Node2D`, `Control` UI, Audio rigs, VFX, projectiles, enemies, pickups…
- **Designer-driven level layouts (Harvest Cache)** — place placeholder instances anywhere under a channel in the editor. At `ready()` their global transforms are cached, the placeholders are freed, and pooled instances are deployed in their exact spots.
- **Checkpoint / level reset** — one call (or one inspector tick) drains every active entity of a channel back to dormant stock and re-deploys the original harvested layout.
- **Warm start (pre-allocation)** — per-channel `pre_alloc_size` builds dormant stock at load time; zero instantiation during gameplay bursts.
- **Dynamic chunk expansion** — if a channel runs dry at runtime, a `+32` instance block is allocated automatically (with a warning) so gameplay never stalls on a miss.
- **Zero-allocation steady state** — despawned entities return to the dormant matrix and are reused; after saturation no new instances are created (verified: 9 400+ spawns served by 500 instances).
- **Channel-scoped tree encapsulation** — active *and* dormant entities are parented under their own `NativePoolChannel` container node, keeping the scene tree organized exactly where the level designer structured it.
- **Safe state toggling** — spawn/despawn flip processing flags and root visibility only; child nodes keep the visibility authored in their `.tscn`.
- **Dead-reference hardening** — freed/invalid instances are skipped automatically; double-despawn is a safe no-op.
- **Scene-teardown memory flush** — leaving a scene queue-frees all pooled entities and clears every registry (no cross-level leaks).
- **Inspector debug controls** — one-shot bool triggers for diagnostics print and channel reset, usable mid-game without code.
- **Script-language friendly** — the full API is callable from GDScript, C#, or anything else that can call Godot methods.

---

## 🧩 The Two Nodes

### `UniversalNativePool` (extends `Node`)

The master controller. Add **one per scene**. It discovers its channels at `ready()`, owns all registries, and exposes the scripting API.

| Inspector property | Type | Purpose |
|---|---|---|
| `Trigger Pool Diagnostic Print` | `bool` (one-shot) | Tick while playing → prints the per-channel readout next frame, then auto-unchecks. |
| `Target Debug Channel Name` | `String` | Channel key used by the reset trigger (e.g. `coins`). |
| `Trigger Channel Reset Now` | `bool` (one-shot) | Tick while playing → checkpoint-resets the named channel next frame, then auto-unchecks. |

### `NativePoolChannel` (extends `Node`)

A pooling channel. Must be a **direct child** of the pool. Its **node name is its channel key** in all script calls.

| Inspector property | Type | Purpose |
|---|---|---|
| `Blueprint Scene` | `PackedScene` | The scene to pool (bullet, coin, enemy, UI popup…). |
| `Pre Alloc Size` | `int` (0–50000) | Dormant stock built at load. `0` = lazy allocation via chunks only. |

---

## 🌳 Node Hierarchy Usage

```
LevelRoot (Node3D / Node2D / any)
└── UniversalNativePool          ← one per scene
    ├── bullets                  ← NativePoolChannel (blueprint = bullet.tscn, pre_alloc = 500)
    ├── coins                    ← NativePoolChannel (blueprint = coin.tscn,  pre_alloc = 0)
    │   ├── Coin                 ← editor placeholders (any Node3D/Node2D)
    │   ├── Coin2                ←   harvested at ready(), then freed
    │   └── Coin3                ←   re-deployed from the pool on reset
    └── enemies                  ← NativePoolChannel (pre-placed enemies OR wave-spawned)
```

**Rules & conventions**

1. Channels are **direct children** of the pool; the channel's **node name = channel key** (`"bullets"`, case-sensitive).
2. **Placeholders** = ordinary scene instances you position in the editor under a channel. At runtime they are harvested (global transform cached), `queue_free()`'d, and replaced by pooled instances at the same spots.
3. Channels with **no placeholders** (e.g. `bullets`) are pure spawn/despawn channels — resets simply dormancy everything.
4. Channels with **placeholders but `pre_alloc_size = 0`** still work: the first deploy triggers chunk allocation.
5. The pool must exist in the tree at `ready()`; channels added later at runtime are **not** registered.
6. Keep the pool **per-scene**, not in an autoload (its teardown flush and one-time harvest are scene-scoped by design).
7. Blueprint roots should be `Node3D` or `Node2D` to receive position/visibility handling; other root types pool fine but are not transformed/shown by the pool.
8. Pooled entities live under their channel node at runtime — inspect `bullets/`, `coins/` in the Remote scene tree to see dormant + active instances.

---

## 📦 Installation

### Option A — Demo project (as released)
Open the packaged Godot project directly. The extension binaries, `.gdextension` file and demo scene (`Main.tscn` with pool, channels, placeholders and `RadialSpawner`) are already wired up. Press **Play**.

### Option B — Extension only, into your own project
1. Copy `bin/` (native libraries) and `gdo_extensions.gdextension` into your project root (keep their relative paths).
2. Ensure the `.gdextension` matches your setup:

   ```ini
   [configuration]
   entry_symbol = "gdext_rust_init"
   compatibility_minimum = "4.7"
   compatibility_maximum = "4.8"

   [libraries]
   windows.debug.x86_64   = "res://bin/gdo_extensions.dll"
   windows.release.x86_64 = "res://bin/gdo_extensions.dll"
   ```

3. Restart Godot (4.7 or 4.8). `UniversalNativePool` and `NativePoolChannel` appear in the *Create New Node* dialog.

### Build from source
```bash
cargo build --release          # Rust 2024 edition, gdext 0.5.5
# copy target/release/gdo_extensions.dll → res://bin/
```

---

## 🎛 Editor Workflow (no code required)

1. Add a `UniversalNativePool` node to your level.
2. Add `NativePoolChannel` children; name them (`bullets`, `coins`, `enemies`…).
3. Assign each channel's **Blueprint Scene** and **Pre Alloc Size**.
4. For level-layout channels, drop placeholder instances under the channel and position them.
5. Play: placeholders are swapped for pooled instances at the same transforms.
6. Mid-game, use the pool's inspector triggers (diagnostics / reset) or the script API below.

---

##  Scripting API Reference

All methods are exposed to Godot (`#[func]`). Channel keys are channel **node names**.

### `UniversalNativePool` methods

| Signature (Godot-side) | Returns | Description |
|---|---|---|
| `spawn(key: String, global_position: Vector3) -> Node` | instance or `null` | Activates one dormant entity, places it at `global_position`, shows it, registers it active. `null` if the channel doesn't exist. For `Node2D` blueprints, `x/y` are used. |
| `despawn(key: String, entity: Node) -> void` | — | Hides + dormancies the entity and returns it to the channel's stock for reuse. Idempotent; ignores invalid or already-dormant entities. |
| `reset_channel(key: String) -> void` | — | Checkpoint reset: all active entities → dormant, then the harvested placeholder layout is re-deployed. |
| `get_active_count(key: String) -> int` | count | Number of currently active (in-the-wild) entities on a channel. For HUDs, debug, wave logic. |

### Properties settable from scripts

| Property | On | Notes |
|---|---|---|
| `trigger_pool_diagnostic_print` | pool | One-shot: prints the readout next frame, auto-resets. |
| `target_debug_channel_name` | pool | Parameter for the reset trigger. |
| `trigger_channel_reset_now` | pool | One-shot reset trigger, auto-resets. |
| `blueprint_scene`, `pre_alloc_size` | channel | Design-time configuration (read at `ready()`). |

---

### GDScript examples (fine-grained)

```gdscript
extends Node3D
## A gun that fires pooled bullets from a muzzle point.

@export var pool_path: NodePath = ^"%UniversalNativePool"   # or a full path
@export var muzzle: Marker3D
@export var fire_rate: float = 20.0

var pool: Node
var _live: Array[Node] = []          # entities we handed out
var _acc: float = 0.0

func _ready() -> void:
    pool = get_node(pool_path)

func _process(delta: float) -> void:
    _acc += fire_rate * delta
    while _acc >= 1.0:
        _acc -= 1.0
        _fire()

func _fire() -> void:
    # 1) SPAWN — returns Node or null
    var bullet: Node = pool.spawn("bullets", muzzle.global_position)
    if bullet == null:
        push_warning("no bullets channel?")
        return

    # 2) Orient it (the pool only positions + shows; gameplay is yours)
    var b3d: Node3D = bullet as Node3D
    if b3d:
        b3d.look_at(b3d.global_position - global_transform.basis.z, Vector3.UP)

    _live.append(bullet)

    # 3) Recycle later — e.g. after a lifespan
    get_tree().create_timer(2.0).timeout.connect(_recycle.bind(bullet))

func _recycle(bullet: Node) -> void:
    if is_instance_valid(bullet):
        pool.despawn("bullets", bullet)   # 4) DESPAWN — back to dormant stock
        _live.erase(bullet)

func _unhandled_input(event: InputEvent) -> void:
    if event.is_action_pressed("ui_accept"):
        # 5) Counts, resets and debug triggers
        print("active bullets: ", pool.get_active_count("bullets"))
        pool.reset_channel("coins")                    # checkpoint-reset the coin layout
        pool.target_debug_channel_name = "coins"
        pool.trigger_channel_reset_now = true          # same reset via inspector path
        pool.trigger_pool_diagnostic_print = true      # dump Dormant/Active table
```

### C# examples (fine-grained)

```csharp
using Godot;

public partial class Gun : Node3D
{
    private Node _pool = null!;

    public override void _Ready()
        => _pool = GetNode<Node>("%UniversalNativePool");

    public override void _Process(double delta)
    {
        if (Input.IsMouseButtonPressed(MouseButton.Left))
            Fire();
    }

    private void Fire()
    {
        // spawn(key, global_position) -> Variant (Object or Nil)
        Variant result = _pool.Call("spawn", "bullets", GlobalPosition);
        if (result.AsGodotObject() is Node3D bullet)
        {
            bullet.LookAt(bullet.GlobalPosition - GlobalTransform.Basis.Z, Vector3.Up);

            // ... later, recycle:
            // _pool.Call("despawn", "bullets", bullet);
        }
    }

    private void DebugDump()
    {
        int active = _pool.Call("get_active_count", "bullets").AsInt32();
        GD.Print($"active bullets: {active}");

        _pool.Call("reset_channel", "coins");
        _pool.Set("target_debug_channel_name", "coins");
        _pool.Set("trigger_channel_reset_now", true);
        _pool.Set("trigger_pool_diagnostic_print", true);
    }
}
```

> GDExtension classes are registered in ClassDB, so any language that can call methods by name works. Always null-check `spawn()` results.

---

## 🩺 Diagnostics Readout

Trigger via inspector checkbox, `trigger_pool_diagnostic_print = true`, or keybind in the demo harness:

```
--- UNIVERSAL NATIVE POOL DIAGNOSTIC READOUT ---
 -> Channel [bullets]: Dormant Matrix: 300, Active Wild: 200, Level Layout Restarts Cached: 0
 -> Channel [coins]:   Dormant Matrix: 24,  Active Wild: 8,   Level Layout Restarts Cached: 8
------------------------------------------------
```

- **Dormant Matrix** — recycled/pre-allocated stock ready to spawn (hidden, processing off).
- **Active Wild** — entities currently in gameplay.
- **Level Layout Restarts Cached** — harvested placeholder transforms stored for resets.

A runtime warning `Channel 'x' hit capacity! Executing chunk allocation block (+32 instances)...` means the channel exceeded its stock once; if it repeats forever, your concurrent load exceeds stock — raise `pre_alloc_size` or shorten entity lifetimes.

---

## ⚠️ Gotchas / FAQ

- **`spawn()` returns null** → channel name typo, channel not a direct child of the pool, or no blueprint assigned.
- **Placeholders visible in editor, pooled instances at runtime** → intended; the swap happens at `ready()`.
- **Entity children keep their authored visibility** → the pool toggles only the blueprint root's visibility; design your `.tscn` accordingly.
- **Reset on a placeholder-less channel** → everything just goes dormant (nothing to re-deploy). Expected.
- **Main thread only** → like all Godot node APIs, call pool methods from the main thread.
- **One pool per scene** → the teardown flush and harvest are scene-scoped by design.

---

## 🗂 Repository Contents

- `rust_extension/gdo_extensions/` — Rust gdext crate (`src/lib.rs`).
- `bin/` — compiled native libraries.
- `gdo_extensions.gdextension` — loader manifest.
- Root Godot demo project — pool + channels + placeholders + `RadialSpawner` stress harness (waves of 200/500/600, continuous spray, recycle testing).

## 📄 License

<!-- TODO: add your license file and reference it here, e.g. MIT -->


USED GOOGLE SEARCH AI to make the code
partial cleanup with qwen
<img width="877" height="955" alt="image" src="https://github.com/user-attachments/assets/48ff3905-736a-4de9-9629-05dfb00c9ebc" />


---

*Built with [godot-rust / gdext](https://github.com/godot-rust/gdext) 0.5.5 · Targets Godot 4.7 – 4.8 · Rust edition 2024*
