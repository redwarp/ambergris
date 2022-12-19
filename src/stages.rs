use bevy::prelude::{Plugin, StageLabel, SystemStage};

#[derive(StageLabel)]
pub enum UpdateStages {
    UpdateMap,
}

pub struct StagesPlugin;

impl Plugin for StagesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_stage_after(
            bevy::app::CoreStage::Update,
            UpdateStages::UpdateMap,
            SystemStage::parallel(),
        );
    }
}
