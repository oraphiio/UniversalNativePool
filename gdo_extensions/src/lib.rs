use godot::prelude::*;
use std::collections::HashMap;

// =============================================================================
// GDEXTENSION HANDSHAKE CONFIGURATION
// =============================================================================
struct GDExtensionEntry;

#[gdextension]
unsafe impl ExtensionLibrary for GDExtensionEntry {}

// =============================================================================
// NODE 1: THE CHILD ARCHETYPE SPECIFICATION NODE
// =============================================================================
/// This node represents an individual asset category folder in the scene tree hierarchy.
/// Inherits from generic Node to cleanly support both Node2D and Node3D asset arrays.
#[derive(GodotClass)]
#[class(base=Node)]
pub struct NativePoolArchetype {
    base: Base<Node>,

    /// The visual prefab file layout configuration resource (.tscn)
    #[export]
    pub blueprint_scene: Option<Gd<PackedScene>>,

    /// The total target storage capacity to allocate up front at boot.
    /// If left at 0, this specific channel shifts to runtime lazy loading growth!
    #[export]
    pub pre_alloc_size: i32,
}

#[godot_api]
impl INode for NativePoolArchetype {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            blueprint_scene: None,
            pre_alloc_size: 0,
        }
    }
}

// =============================================================================
// NODE 2: THE MASTER DICTIONARY SYSTEM CONTROL POOL
// =============================================================================
#[derive(GodotClass)]
#[class(base=Node)]
pub struct UniversalNativePool {
    base: Base<Node>,

    /// Master Cache Mapping: String Key -> Vector array of resting, unmanaged Node asset instances
    pool_registry: HashMap<String, Vec<Gd<Node>>>,

    /// Master Reference Mapping: String Key -> Scene asset configuration profile recipes
    scene_blueprints: HashMap<String, Gd<PackedScene>>,
}

#[godot_api]
impl INode for UniversalNativePool {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            pool_registry: HashMap::new(),
            scene_blueprints: HashMap::new(),
        }
    }

    fn ready(&mut self) {
        godot_print!("UniversalNativePool v1.0 [Modular Core Pipeline Active]");
        self.initialize_and_harvest_archetypes();
    }
}

#[godot_api]
impl UniversalNativePool {
    /// Internal automated initialization loop.
    /// Scans its immediate child tree for NativePoolArchetype nodes to build memory channels.
    fn initialize_and_harvest_archetypes(&mut self) {
        let children = self.base().get_children();

        for child in children.iter_shared() {
            // Safe downcast verification: Is this child a custom NativePoolArchetype?
            if let Ok(archetype_node) = child.try_cast::<NativePoolArchetype>() {
                // Derives the tracking dictionary key string directly from the node's custom name!
                let pool_key = archetype_node.get_name().to_string();
                let (blueprint_opt, size) = {
                    let bind = archetype_node.bind();
                    (bind.blueprint_scene.clone(), bind.pre_alloc_size)
                };

                let Some(blueprint) = blueprint_opt else {
                    godot_error!("UniversalNativePool Error: Archetype Node '{}' has no .tscn blueprint assigned!", pool_key);
                    continue;
                };

                if !blueprint.is_instance_valid() {
                    godot_error!("UniversalNativePool Error: Archetype Node '{}' has no valid .tscn blueprint assigned!", pool_key);
                    continue;
                }

                // Register blueprint profile memory
                self.scene_blueprints
                    .insert(pool_key.clone(), blueprint.clone());

                let mut storage_vector = Vec::new();
                if size > 0 {
                    storage_vector.reserve(size as usize);
                    for _ in 0..size {
                        if let Some(instance) = blueprint.instantiate() {
                            let mut entity = instance;

                            // Put entity into a low-overhead deep dormancy state
                            entity.set_process(false);
                            entity.set_physics_process(false);

                            // Safe visual deactivation checks across both 2D and 3D node variants
                            if let Ok(mut spatial_3d) = entity.clone().try_cast::<Node3D>() {
                                spatial_3d.set_visible(false);
                            } else if let Ok(mut canvas_2d) = entity.clone().try_cast::<Node2D>() {
                                canvas_2d.set_visible(false);
                            }

                            // Secure node handle nested under the active structural layout tree root
                            self.base_mut().add_child(&entity);

                            storage_vector.push(entity);
                        }
                    }
                }

                self.pool_registry.insert(pool_key.clone(), storage_vector);
                godot_print!(
                    "UniversalNativePool: Channel '{}' pre-allocated with {} elements.",
                    pool_key,
                    size
                );
            }
        }
    }

    // =============================================================================
    // THE SEAMLESS PROGRAMMER SCRIPT BRIDGE API INTERFACE
    // =============================================================================

    /// Click-Clack Spawning Bridge: Pulls a dynamic or pre-allocated object handle instantly
    #[func]
    pub fn spawn(&mut self, key: String, global_position: Vector3) -> Option<Gd<Node>> {
        if !self.pool_registry.contains_key(&key) {
            godot_error!(
                "UniversalNativePool API: Request key '{}' does not exist in registry map!",
                key
            );
            return None;
        }

        // Automatic Emergency Overflow Strategy: Lazy load expansion if vector arrays run dry
        let is_empty = self.pool_registry.get(&key).is_none_or(|v| v.is_empty());
        if is_empty {
            if let Some(blueprint) = self.scene_blueprints.get(&key).cloned() {
                if let Some(instance) = blueprint.instantiate() {
                    let mut entity = instance;
                    entity.set_process(false);
                    entity.set_physics_process(false);

                    if let Ok(mut spatial_3d) = entity.clone().try_cast::<Node3D>() {
                        spatial_3d.set_visible(false);
                    } else if let Ok(mut canvas_2d) = entity.clone().try_cast::<Node2D>() {
                        canvas_2d.set_visible(false);
                    }

                    self.base_mut().add_child(&entity);

                    if let Some(vector) = self.pool_registry.get_mut(&key) {
                        vector.push(entity);
                    }
                }
            }
        }

        // Fetch handle instance tracking elements from the unmanaged caching backend arrays
        if let Some(vector) = self.pool_registry.get_mut(&key) {
            if let Some(mut entity) = vector.pop() {
                // Route target global transform coordinates depending on spatial orientation profile
                if let Ok(mut spatial_3d) = entity.clone().try_cast::<Node3D>() {
                    spatial_3d.set_global_position(global_position);
                    spatial_3d.set_visible(true);
                } else if let Ok(mut canvas_2d) = entity.clone().try_cast::<Node2D>() {
                    // Truncates Vector3 position elements seamlessly for standard 2D view layouts
                    canvas_2d
                        .set_global_position(Vector2::new(global_position.x, global_position.y));
                    canvas_2d.set_visible(true);
                }

                // Awakening entity logic sweeps
                entity.set_process(true);
                entity.set_physics_process(true);

                return Some(entity);
            }
        }

        None
    }

    /// Click-Clack Recycling Bridge: Pushes an active element straight back down into unmanaged caches
    #[func]
    pub fn despawn(&mut self, key: String, mut entity: Gd<Node>) {
        if let Some(vector) = self.pool_registry.get_mut(&key) {
            // Put processing logic states back to sleep safely
            entity.set_process(false);
            entity.set_physics_process(false);

            if let Ok(mut spatial_3d) = entity.clone().try_cast::<Node3D>() {
                spatial_3d.set_visible(false);
            } else if let Ok(mut canvas_2d) = entity.clone().try_cast::<Node2D>() {
                canvas_2d.set_visible(false);
            }

            // Slide reference back onto structural unmanaged vector array registers
            vector.push(entity);
        } else {
            godot_error!(
                "UniversalNativePool API: Direct Despawn failure. Invalid target key channel '{}'",
                key
            );
        }
    }
}
