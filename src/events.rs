use bevy::ecs::{entity::Entity, event::EntityEvent};

use crate::{assets::LdtkTilesetRef, ldtk::EntityInstance};

/// Triggered when an LDtk entity instance has been spawned.
#[derive(EntityEvent, Clone)]
pub struct LdtkEntitySpawned {
    pub entity: Entity,
    /// The LDtk entity instance this entity was spawned from.
    pub instance: EntityInstance,
    /// The tileset of the entity's editor visual, if it has one.
    pub tileset: Option<LdtkTilesetRef>,
}
