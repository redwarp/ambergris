use crate::{colors, components::*, game::Ai};

pub enum MonsterType {
    Orc,
    Troll,
}

pub fn monster(
    monster_type: MonsterType,
    x: i32,
    y: i32,
) -> (Monster, Coordinates, Body, CombatStats) {
    match monster_type {
        MonsterType::Orc => orc(x, y),
        MonsterType::Troll => troll(x, y),
    }
}

fn orc(x: i32, y: i32) -> (Monster, Coordinates, Body, CombatStats) {
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
    (
        Monster {
            ai: Ai::Basic,
            speed: 900,
            tick: 0,
        },
        Coordinates::new(x, y),
        body,
        combat_stats,
    )
}

fn troll(x: i32, y: i32) -> (Monster, Coordinates, Body, CombatStats) {
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
    (
        Monster {
            ai: Ai::Basic,
            speed: 1100,
            tick: 0,
        },
        Coordinates { x, y },
        body,
        combat_stats,
    )
}

pub fn player(x: i32, y: i32) -> (Player, Coordinates, Body, CombatStats) {
    (
        Player { speed: 1000 },
        Coordinates { x, y },
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
    )
}

pub fn potion(x: i32, y: i32) -> (Item, Coordinates, Body, ProvidesHealing, Consumable) {
    (
        Item {},
        Coordinates { x, y },
        Body {
            name: "potion".to_string(),
            blocking: false,
            char: 'i',
            color: colors::PURPLE,
        },
        ProvidesHealing { heal_amount: 5 },
        Consumable {},
    )
}

pub fn invisibility_potion(x: i32, y: i32) -> (Item, Coordinates, Body, Consumable) {
    (
        Item {},
        Coordinates { x, y },
        Body {
            name: "invisibility potion".to_string(),
            blocking: false,
            char: 'i',
            color: colors::PURPLE,
        },
        Consumable {},
    )
}
