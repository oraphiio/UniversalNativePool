use godot::prelude::*;
use std::collections::HashMap;

// =============================================================================
// GDEXTENSION HANDSHAKE CONFIGURATION
// =============================================================================
struct UniversalPoolLibrary;

// SUCCESS: The modern macro parameter uses an unquoted token that
// maps directly to entry_symbol inside your .gdextension file!
#[gdextension(entry_symbol = gdext_rust_init)]
unsafe impl ExtensionLibrary for UniversalPoolLibrary {}

// =============================================================================
// HELPER FUNCTIONS FOR SAFE CASTING & STATE MANAGEMENT
// =============================================================================

/// Safely deactivates an entity and all its child components recursively.
fn deactivate_entity(mut node: Gd<Node>) {
    node.set_process(false);
    node.set_physics_process(false);
    node.set_process_internal(false);
    node.set_physics_process_internal(false);

    if let Ok(mut node_3d) = node.clone().try_cast::<Node3D>() {
        node_3d.set_visible(false);
    } else if let Ok(mut node_2d) = node.clone().try_cast::<Node2D>() {
        node_2d.set_visible(false);
    }
}

/// Safely reactivates an entity and all its child components recursively.
fn activate_entity(mut node: Gd<Node>) {
    node.set_process(true);
    node.set_physics_process(true);
    node.set_process_internal(true);
    node.set_physics_process_internal(true);

    if let Ok(mut node_3d) = node.clone().try_cast::<Node3D>() {
        node_3d.set_visible(true);
    } else if let Ok(mut node_2d) = node.clone().try_cast::<Node2D>() {
        node_2d.set_visible(true);
    }
}

/// Applies harvested transform data to a newly spawned node.
fn apply_harvested_transform(entity: &Gd<Node>, data: &HarvestedSpatialData) {
    let root_node = entity.clone();

    match data {
        HarvestedSpatialData::Spatial3D { transform } => {
            if let Ok(mut node_3d) = root_node.try_cast::<Node3D>() {
                node_3d.set_global_transform(*transform);
                node_3d.set_visible(true);

                // VERBOSE DEBUG HOOK: Log the exact world placement where the node is positioned
                let origin = transform.origin;
                godot_print!(
                    "🔧 [RUST DEBUG]: Re-deploying spatial node '{}' to World Positions -> X: {:.2}, Y: {:.2}, Z: {:.2}",
                    entity.get_name(),
                    origin.x,
                    origin.y,
                    origin.z
                );
            }
        }
        HarvestedSpatialData::Canvas2D { transform } => {
            if let Ok(mut node_2d) = root_node.try_cast::<Node2D>() {
                node_2d.set_global_transform(*transform);
                node_2d.set_visible(true);
            }
        }
    }
}

// =============================================================================
// NODE 1: THE CHILD CHANNEL SPECIFICATION NODE
// =============================================================================
#[derive(GodotClass)]
#[class(base=Node)]
pub struct NativePoolChannel {
    base: Base<Node>,

    #[export]
    pub blueprint_scene: Option<Gd<PackedScene>>,

    #[export(range = (0.0, 50000.0))]
    pub pre_alloc_size: i32,
}

#[godot_api]
impl INode for NativePoolChannel {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            blueprint_scene: None,
            pre_alloc_size: 0,
        }
    }
}

// =============================================================================
// INTERNAL HARVEST TRANSFORMS MEMORY ARCHITECTURE
// =============================================================================
#[derive(Clone, Copy)]
enum HarvestedSpatialData {
    Spatial3D { transform: Transform3D },
    Canvas2D { transform: Transform2D },
}

// =============================================================================
// NODE 2: THE MASTER DICTIONARY SYSTEM CONTROL POOL
// =============================================================================
#[derive(GodotClass)]
#[class(base=Node)]
pub struct UniversalNativePool {
    base: Base<Node>,

    pool_registry: HashMap<String, Vec<Gd<Node>>>,
    active_registry: HashMap<String, Vec<Gd<Node>>>,
    scene_blueprints: HashMap<String, Gd<PackedScene>>,
    harvest_cache: HashMap<String, Vec<HarvestedSpatialData>>,

    #[export]
    pub trigger_pool_diagnostic_print: bool,

    // GString is the canonical type for Godot-facing exported strings
    #[export]
    pub target_debug_channel_name: GString,

    #[export]
    pub trigger_channel_reset_now: bool,
}

#[godot_api]
impl INode for UniversalNativePool {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            pool_registry: HashMap::new(),
            active_registry: HashMap::new(),
            scene_blueprints: HashMap::new(),
            harvest_cache: HashMap::new(),
            trigger_pool_diagnostic_print: false,
            target_debug_channel_name: GString::new(),
            trigger_channel_reset_now: false,
        }
    }

    fn ready(&mut self) {
        godot_print!("UniversalNativePool v1.03 [Harvest & Checkpoint Reset Engine Active]");
        self.initialize_and_harvest_channels();
    }

    fn process(&mut self, _delta: f64) {
        if self.trigger_pool_diagnostic_print {
            self.trigger_pool_diagnostic_print = false;
            self.print_pool_diagnostics();
        }

        if self.trigger_channel_reset_now {
            self.trigger_channel_reset_now = false;
            let channel = self.target_debug_channel_name.to_string();

            if !channel.is_empty() {
                self.reset_channel_internal(channel);
            } else {
                godot_error!(
                    "UniversalNativePool Inspector Error: Enter a channel name in 'target_debug_channel_name'!"
                );
            }
        }
    }

    // STABILIZATION 1: LEVEL TEARDOWN MEMORY FLUSH
    // Automatically triggers when changing scenes, preventing cross-level memory leaks!

    fn exit_tree(&mut self) {
        godot_print!("UniversalNativePool: Cleaning up memory footprint for scene transition...");

        for (_, vector) in self.pool_registry.drain() {
            for mut entity in vector {
                if entity.is_instance_valid() {
                    entity.queue_free();
                }
            }
        }
        for (_, vector) in self.active_registry.drain() {
            for mut entity in vector {
                if entity.is_instance_valid() {
                    entity.queue_free();
                }
            }
        }
        self.scene_blueprints.clear();
        self.harvest_cache.clear();
    }
}

#[godot_api]
impl UniversalNativePool {
    fn initialize_and_harvest_channels(&mut self) {
        self.base_mut().set_process(true);
        let children = self.base().get_children();

        for child in children.iter_shared() {
            let channel_container: Gd<Node> = child.clone();
            let Ok(channel_node) = child.try_cast::<NativePoolChannel>() else {
                continue;
            };

            let pool_key = channel_node.get_name().to_string();

            let (blueprint_opt, size) = {
                let bind = channel_node.bind();
                (bind.blueprint_scene.clone(), bind.pre_alloc_size)
            };

            let Some(blueprint) = blueprint_opt else {
                continue;
            };

            if !blueprint.is_instance_valid() {
                continue;
            }

            self.scene_blueprints
                .insert(pool_key.clone(), blueprint.clone());
            self.pool_registry.insert(pool_key.clone(), Vec::new());
            self.active_registry.insert(pool_key.clone(), Vec::new());

            // Harvest placeholders FIRST (before pre-allocation)
            let placeholders = channel_node.get_children();
            let mut collected_transforms = Vec::new();

            for placeholder in placeholders.iter_shared() {
                if let Ok(spatial_3d) = placeholder.clone().try_cast::<Node3D>() {
                    collected_transforms.push(HarvestedSpatialData::Spatial3D {
                        transform: spatial_3d.get_global_transform(),
                    });
                } else if let Ok(canvas_2d) = placeholder.clone().try_cast::<Node2D>() {
                    collected_transforms.push(HarvestedSpatialData::Canvas2D {
                        transform: canvas_2d.get_global_transform(),
                    });
                }
                let mut removable = placeholder;
                removable.queue_free();
            }
            // Pre-allocation block (Borrow-checker safe)
            // Pre-allocation block (Parented safely to channel container branch node)
            if size > 0 {
                let mut allocated = Vec::with_capacity(size as usize);
                for _ in 0..size {
                    if let Some(entity) = blueprint.instantiate() {
                        deactivate_entity(entity.clone());
                        let mut mut_channel = channel_node.clone();
                        mut_channel.add_child(&entity);
                        allocated.push(entity);
                    }
                }
                if let Some(pool) = self.pool_registry.get_mut(&pool_key) {
                    pool.extend(allocated);
                }
            }

            let total_harvested = collected_transforms.len();
            self.harvest_cache
                .insert(pool_key.clone(), collected_transforms.clone());

            // Apply harvested layouts to active pool nodes parented under the channel container
            for spatial_data in collected_transforms {
                if let Some(active_node) =
                    self.spawn_from_pool_internal(&pool_key, &channel_container)
                {
                    apply_harvested_transform(&active_node, &spatial_data);

                    if let Some(active_vec) = self.active_registry.get_mut(&pool_key) {
                        active_vec.push(active_node);
                    }
                }
            }

            godot_print!(
                "UniversalNativePool: Channel '{}' compiled and active. Pre-allocated: {}, Harvested: {}.",
                pool_key,
                size,
                total_harvested
            );
        }
    }

    fn spawn_from_pool_internal(&mut self, key: &str, channel_node: &Gd<Node>) -> Option<Gd<Node>> {
        if !self.pool_registry.contains_key(key) {
            return None;
        }

        let pool_empty = self.pool_registry.get(key).map_or(true, |v| v.is_empty());

        // Dynamic Chunk Expansion (+32 Block allocation step parented to the channel node context)
        if pool_empty {
            if let Some(blueprint) = self.scene_blueprints.get(key).cloned() {
                godot_warn!(
                    "UniversalNativePool: Channel '{}' hit capacity! Executing chunk allocation block (+32 instances)...",
                    key
                );
                let mut temp_allocated = Vec::with_capacity(32);
                for _ in 0..32 {
                    if let Some(entity) = blueprint.instantiate() {
                        deactivate_entity(entity.clone());
                        // Parent explicitly to the channel container node path block
                        let mut mut_channel = channel_node.clone();
                        mut_channel.add_child(&entity);
                        temp_allocated.push(entity);
                    }
                }

                if let Some(pool) = self.pool_registry.get_mut(key) {
                    pool.extend(temp_allocated);
                }
            }
        }

        if let Some(pool) = self.pool_registry.get_mut(key) {
            while let Some(entity) = pool.pop() {
                if entity.is_instance_valid() {
                    activate_entity(entity.clone());
                    return Some(entity);
                }
            }
        }

        None
    }

    #[func]
    pub fn spawn(&mut self, key: GString, global_position: Vector3) -> Option<Gd<Node>> {
        let key_str = key.to_string();
        if !self.pool_registry.contains_key(&key_str) {
            return None;
        }

        // Fetch the active container node branch dynamically out of the children tree
        let channel_node = self.base().get_node_or_null(&key_str)?;

        if let Some(entity) = self.spawn_from_pool_internal(&key_str, &channel_node) {
            if let Ok(mut s3d) = entity.clone().try_cast::<Node3D>() {
                s3d.set_global_position(global_position);
                s3d.set_visible(true);
            } else if let Ok(mut c2d) = entity.clone().try_cast::<Node2D>() {
                c2d.set_global_position(Vector2::new(global_position.x, global_position.y));
                c2d.set_visible(true);
            }

            if let Some(active_vec) = self.active_registry.get_mut(&key_str) {
                active_vec.push(entity.clone());
            }
            return Some(entity);
        }
        None
    }

    #[func]
    pub fn despawn(&mut self, key: GString, entity: Gd<Node>) {
        let key_str = key.to_string();

        if !entity.is_instance_valid() {
            return;
        }

        if let Some(vector) = self.pool_registry.get_mut(&key_str) {
            if vector.contains(&entity) {
                return; // Already dormant
            }

            deactivate_entity(entity.clone());

            if let Some(active_vec) = self.active_registry.get_mut(&key_str) {
                if let Some(index) = active_vec.iter().position(|x| *x == entity) {
                    active_vec.swap_remove(index);
                }
            }
            vector.push(entity);
        }
    }

    // Internal logic for reset to avoid GString borrowing issues in process()
    fn reset_channel_internal(&mut self, key: String) {
        if !self.pool_registry.contains_key(&key) {
            return;
        }

        let active_nodes: Vec<Gd<Node>> = self
            .active_registry
            .get_mut(&key)
            .map(|v| v.drain(..).collect())
            .unwrap_or_default();

        if let Some(dormant_vec) = self.pool_registry.get_mut(&key) {
            for entity in active_nodes {
                if !entity.is_instance_valid() {
                    continue;
                }
                deactivate_entity(entity.clone());
                if !dormant_vec.contains(&entity) {
                    dormant_vec.push(entity);
                }
            }
        }

        if let Some(cached_transforms) = self.harvest_cache.get(&key).cloned() {
            let Some(channel_node) = self.base().get_node_or_null(&key) else {
                godot_error!(
                    "UniversalNativePool: reset aborted - channel node '{}' not found in tree.",
                    key
                );
                return;
            };
            for spatial_data in cached_transforms {
                if let Some(active_node) = self.spawn_from_pool_internal(&key, &channel_node) {
                    apply_harvested_transform(&active_node, &spatial_data);

                    if let Some(active_vec) = self.active_registry.get_mut(&key) {
                        active_vec.push(active_node);
                    }
                }
            }
        }

        godot_print!(
            "UniversalNativePool: Channel '{}' successfully reset to layout targets.",
            key
        );
    }

    #[func]
    pub fn reset_channel(&mut self, key: GString) {
        self.reset_channel_internal(key.to_string());
    }

    #[func]
    pub fn get_active_count(&self, key: GString) -> i32 {
        let key_str = key.to_string();
        self.active_registry
            .get(&key_str)
            .map_or(0, |v| v.len() as i32)
    }

    fn print_pool_diagnostics(&self) {
        godot_print!("--- UNIVERSAL NATIVE POOL DIAGNOSTIC READOUT ---");
        for (key, dormant_vec) in &self.pool_registry {
            let active_count = self.active_registry.get(key).map_or(0, |v| v.len());
            let harvest_count = self.harvest_cache.get(key).map_or(0, |v| v.len());
            godot_print!(
                " -> Channel [{}]: Dormant Matrix: {}, Active Wild: {}, Level Layout Restarts Cached: {}",
                key,
                dormant_vec.len(),
                active_count,
                harvest_count
            );
        }
        godot_print!("------------------------------------------------");
    }
}
