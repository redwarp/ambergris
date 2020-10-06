use crate::{components::*, game::Ai};

pub enum MonsterType {
    Orc,
    Troll,
}

pub fn spawn_monster(monster_type: MonsterType, x: i32, y: i32) -> (Monster, Body, CombatStats) {
    match monster_type {
        MonsterType::Orc => spawn_orc_body(x, y),
        MonsterType::Troll => spawn_troll_body(x, y),
    }
}

fn spawn_orc_body(x: i32, y: i32) -> (Monster, Body, CombatStats) {
    let body = Body {
        name: "orc".into(),
        x,
        y,
        blocking: true,
        char: 'o',
        color: crate::colors::DESATURATED_GREEN,
    };
    let combat_stats = CombatStats {
        max_hp: 12,
        hp: 12,
        defense: 1,
        attack: 3,
    };
    (
        Monster {
            ai: Ai::Basic,
            speed: 900,
            tick: 0,
        },
        body,
        combat_stats,
    )
}

fn spawn_troll_body(x: i32, y: i32) -> (Monster, Body, CombatStats) {
    let body = Body {
        name: "troll".into(),
        x,
        y,
        blocking: true,
        char: 'T',
        color: crate::colors::DARKER_GREEN,
    };
    let combat_stats = CombatStats {
        max_hp: 20,
        hp: 20,
        defense: 1,
        attack: 6,
    };
    (
        Monster {
            ai: Ai::Basic,
            speed: 1100,
            tick: 0,
        },
        body,
        combat_stats,
    )
}

pub fn spawn_player(x: i32, y: i32) -> (Player, Body, CombatStats) {
    (
        Player { speed: 1000 },
        Body {
            name: "player".into(),
            x,
            y,
            blocking: true,
            char: '@',
            color: crate::colors::YELLOW,
        },
        CombatStats {
            max_hp: 30,
            hp: 30,
            attack: 5,
            defense: 2,
        },
    )
}
