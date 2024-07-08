use bevy::prelude::Entity;
use crate::map::attributes::{Hp, Size};

#[derive(Debug, Copy, Clone)]
pub enum CombatantType {
    Player,
    Summon,
    Enemy,
    Ally,
}

pub struct Combatant {
    texture: usize,
    summoner: Option<Entity>,
    summons: Option<Entity>,
    name: String,
    size: Size,
    hp: Hp,
    combatant_type: CombatantType,
}