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
        color: tcod::colors::DESATURATED_GREEN,
    };
    let combat_stats = CombatStats {
        max_hp: 12,
        hp: 12,
        defense: 1,
        attack: 2,
    };
    (Monster { ai: Ai::Basic }, body, combat_stats)
}

fn spawn_troll_body(x: i32, y: i32) -> (Monster, Body, CombatStats) {
    let body = Body {
        name: "troll".into(),
        x,
        y,
        blocking: true,
        char: 'T',
        color: tcod::colors::DARKER_GREEN,
    };
    let combat_stats = CombatStats {
        max_hp: 12,
        hp: 12,
        defense: 1,
        attack: 2,
    };
    (Monster { ai: Ai::Basic }, body, combat_stats)
}

pub fn spawn_player(x: i32, y: i32) -> (Player, Body, CombatStats) {
    (
        Player,
        Body {
            name: "player".into(),
            x,
            y,
            blocking: true,
            char: '@',
            color: tcod::colors::YELLOW,
        },
        CombatStats {
            max_hp: 30,
            hp: 30,
            attack: 5,
            defense: 2,
        },
    )
}
