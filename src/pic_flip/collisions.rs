use bevy::prelude::*;
use std::hash::{Hash, Hasher};

pub fn pair(a: u32, b: u32) -> u32 {
    (a + b) * (a + b + 1) / 2 + b
}

pub fn generate_id(a: u32, b: u32) -> u32 {
    if a < b {
        pair(a, b)
    } else {
        pair(b, a)
    }
}

pub struct Collision {
    pub first_entity: Entity,
    pub second_entity: Entity,
    pub normal: Vec2,
    pub depth: f32,
}

impl Hash for Collision {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.first_entity < self.second_entity {
            self.first_entity.hash(state);
            self.second_entity.hash(state);
        } else {
            self.second_entity.hash(state);
            self.first_entity.hash(state);
        }
    }
}

impl PartialEq<Self> for Collision {
    fn eq(&self, other: &Self) -> bool {
        let first_ordered_pair = if self.first_entity < self.second_entity {
            (self.first_entity, self.second_entity)
        } else {
            (self.second_entity, self.first_entity)
        };

        let other_ordered_pair = if other.first_entity < other.second_entity {
            (other.first_entity, other.second_entity)
        } else {
            (other.second_entity, other.first_entity)
        };

        first_ordered_pair == other_ordered_pair
    }
}

impl Eq for Collision {}
