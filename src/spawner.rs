use legion::{Entity, World};

use crate::{colors, components::*, game::Ai, map::Position, palette};

pub enum MonsterType {
    Orc,
    Troll,
}

pub fn monster(world: &mut World, monster_type: MonsterType, x: i32, y: i32) {
    match monster_type {
        MonsterType::Orc => orc(world, x, y),
        MonsterType::Troll => troll(world, x, y),
    };
}

fn orc(world: &mut World, x: i32, y: i32) {
    let body = Body {
        name: "orc".into(),
        blocking: true,
        char: 'o',
        color: colors::DESATURATED_GREEN,
    };
    let combat_stats = CombatStats {
        max_hp: 12,
        hp: 12,
        defense: 0,
        attack: 3,
    };
    world.push((
        Monster {
            ai: Ai::Basic,
            speed: 900,
            tick: 0,
        },
        Position::new(x, y),
        body,
        combat_stats,
    ));
}

fn troll(world: &mut World, x: i32, y: i32) {
    let body = Body {
        name: "troll".into(),
        blocking: true,
        char: 'T',
        color: colors::DARKER_GREEN,
    };
    let combat_stats = CombatStats {
        max_hp: 20,
        hp: 20,
        defense: 1,
        attack: 4,
    };
    world.push((
        Monster {
            ai: Ai::Basic,
            speed: 1100,
            tick: 0,
        },
        Position { x, y },
        body,
        combat_stats,
    ));
}

pub fn player(world: &mut World, x: i32, y: i32) -> Entity {
    world.push((
        Player { speed: 1000 },
        Position { x, y },
        Body {
            name: "player".to_string(),
            blocking: true,
            char: '@',
            color: colors::YELLOW,
        },
        CombatStats {
            max_hp: 30,
            hp: 30,
            attack: 5,
            defense: 2,
        },
    ))
}

pub fn potion(world: &mut World, x: i32, y: i32) {
    world.push((
        Item {},
        Position { x, y },
        Body {
            name: "potion".to_string(),
            blocking: false,
            char: 'i',
            color: palette::HEALTH,
        },
        ProvidesHealing { heal_amount: 5 },
        Consumable {},
    ));
}

pub fn scroll_of_lightning_bolt(world: &mut World, x: i32, y: i32) {
    world.push((
        Item {},
        Position { x, y },
        Body {
            name: "scroll of lightning bolt".to_string(),
            blocking: false,
            char: '#',
            color: colors::CYAN,
        },
        InflictsDamage { damage: 10 },
        Ranged { range: 7 },
        Consumable {},
    ));
}

pub fn scroll_of_fireball(world: &mut World, x: i32, y: i32) {
    world.push((
        Item {},
        Position { x, y },
        Body {
            name: "scroll of fireball".to_string(),
            blocking: false,
            char: '#',
            color: colors::ORANGE,
        },
        InflictsDamage { damage: 8 },
        Ranged { range: 6 },
        Burst { radius: 1 },
        Consumable {},
    ));
}
