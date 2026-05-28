use crate::model::app_state::AppState;
use sysinfo::Components;

pub struct SensorCollector {
    components: Components,
}

impl SensorCollector {
    pub fn new() -> Self {
        Self {
            components: Components::new_with_refreshed_list(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.components.iter().next().is_none()
    }
    pub fn collect(&mut self, state: &mut AppState) {
        self.components.refresh(true);

        let readings: Vec<(String, Option<f32>, Option<f32>)> = self
            .components
            .iter()
            .map(|c| (c.label().to_string(), c.temperature(), c.critical()))
            .collect();

        state.update(readings);
    }
}