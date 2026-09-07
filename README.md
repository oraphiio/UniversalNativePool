# UniversalNativePool (v1.0) 🚀

A high-performance, modular **GDExtension Object Pool** for Godot 4.7+ compiled natively in **Rust**. It operates entirely in unmanaged, bare-metal memory layers, bypassing Godot's runtime script interpretation and avoiding high-level garbage collection spikes completely.

By utilizing a generic class mapping architecture, a single master node can house, track, and dynamically stream **both 2D and 3D assets** (bullets, actors, foliage, particle emitters) simultaneously under independent string identifiers.

---

## Key Architecture Benefits

* ⚡ **Zero-Allocation Gameplay:** Instantiate thousands of moving elements (like projectiles) without calling the engine's slow object factory during active loops. 
* 🏎️ **O(1) Dictionary Layer:** Uses highly optimized native Rust `HashMap` registers to pop and recycle nodes at microsecond speeds.
* 🌐 **Cross-Language Compatible:** Acts as a foundational engine node addition. It can be called seamlessly by both **GDScript** and **C#** programmers.
* 🛠️ **Modular Inspector Layout:** Designers configure scene blueprints and memory quotas visually in the editor tree without touching code.

---

## Setup Hierarchy (The "Click-Clack" Blueprint)

You do not need to manage complex arrays or setup tags. Simply drop **one** master pool control node into your scene, and add custom archetype children under it for each item variety you wish to manage:

```text
┖╴UniversalNativePool (Master Root Control Node)
    ┠╴PlayerBullet  (NativePoolArchetype Child ➡️ Scene: player_bullet.tscn ➡️ Pre-Alloc: 500)
    ┠╴EnemyMech     (NativePoolArchetype Child ➡️ Scene: desert_tank.tscn   ➡️ Pre-Alloc: 15)
    ┖╴GoldCoin      (NativePoolArchetype Child ➡️ Scene: gold_coin.tscn     ➡️ Pre-Alloc: 0)
```

### The Allocation Rules (Per-Child Control)
* **Pre-Allocation Mode:** Setting a value greater than `0` (e.g., `500`) commands Rust to immediately warm up those instances in memory behind your level's loading screen.
* **Lazy Loading Mode:** Setting the size value to `0` enables dynamic memory tracking. It uses zero overhead at boot and scales up the allocation cache structure automatically on-demand during active gameplay.

---

## Seamless Script API Bridge

The extension exposes a two-method API signature that interfaces directly with your custom wave managers, level triggers, or actor logic loops:

### 1. In GDScript

```gdscript
extends Node3D

@onready var pool = \$UniversalNativePool

func fire_projectile() -> void:
    var target_pos = Vector3(0.0, 1.5, -5.0)
    
    # Click-Clack: Fetch an instance from Rust bare-metal vectors instantly!
    # Hand the key name derived from the Archetype Child Node's name in your scene tree.
    var bullet = pool.spawn("PlayerBullet", target_pos)

func _on_bullet_collision(bullet_instance: Node) -> void:
    # Click-Clack: Cleanly return it right back into the unmanaged memory matrix
    pool.despawn("PlayerBullet", bullet_instance)
```

### 2. In C#

```csharp
using Godot;
using System;

public partial class CombatManager : Node3D
{
    private Node _nativePool;

    public override void _Ready()
    {
        _nativePool = GetNode("UniversalNativePool");
    }

    public void TriggerEnemyReinforcement(Vector3 spawnPosition)
    {
        // The exact same native compiled C-API endpoints connect flawlessly in C#!
        Node enemy = _nativePool.Call("spawn", "EnemyMech", spawnPosition).As<Node>();
    }

    public void RecycleEnemy(Node enemyInstance)
    {
        _nativePool.Call("despawn", "EnemyMech", enemyInstance);
    }
}
```

---

## Installation & Compilation (godot-rust v0.5.5)

1. Drop the `rust_extension` folder containing the `lib.rs` and cargo scripts next to your main Godot project.
2. Compile the native dynamic library binary using the release optimization flag:
   ```powershell
   cargo build --release
   ```
3. Copy the compiled `.dll` (Windows), `.so` (Linux), or `.dylib` (macOS) output file straight into your target Godot destination path:
   ```powershell
   Copy-Item "./target/release/gdo_extensions.dll" "../godot_project/bin/"
   ```
4. Restart your Godot Editor, search for `UniversalNativePool` or `NativePoolArchetype` inside the Node window creation popup, and begin configuring your system layout modules!
