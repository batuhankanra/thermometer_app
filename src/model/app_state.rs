

use std::collections::VecDeque;

pub const HISTORY:  usize = 60;   
pub const TICK_MS:  u64   = 2000; 


#[derive(Clone)]
pub struct SensorReading {
    pub label:    String,
    pub celsius:  Option<f32>,     
    pub critical: Option<f32>,     
    pub max_seen: f32,              
}

impl SensorReading {
    pub fn status(&self) -> ThermalStatus {
        match self.celsius {
            None => ThermalStatus::Unknown,
            Some(t) => {
                let crit = self.critical.unwrap_or(90.0);
                if t >= crit        { ThermalStatus::Critical }
                else if t >= 80.0  { ThermalStatus::Hot      }
                else if t >= 60.0  { ThermalStatus::Warm     }
                else               { ThermalStatus::Cool     }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ThermalStatus {
    Cool,
    Warm,
    Hot,
    Critical,
    Unknown,
}


pub struct AppState {
    pub sensors:     Vec<SensorReading>,
    pub tick:        u64,
    pub should_quit: bool,

    pub selected_idx:  usize,
    pub temp_history:  VecDeque<f64>,

    sensor_max: Vec<f32>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sensors:      vec![],
            tick:         0,
            should_quit:  false,
            selected_idx: 0,
            temp_history: VecDeque::with_capacity(HISTORY),
            sensor_max:   vec![],
        }
    }

    pub fn update(&mut self, readings: Vec<(String, Option<f32>, Option<f32>)>) {
        self.tick += 1;

        if self.sensor_max.len() < readings.len() {
            self.sensor_max.resize(readings.len(), 0.0);
        }

        self.sensors = readings
            .into_iter()
            .enumerate()
            .map(|(i, (label, celsius, critical))| {
                if let Some(t) = celsius {
                    if t > self.sensor_max[i] {
                        self.sensor_max[i] = t;
                    }
                }
                SensorReading {
                    label,
                    celsius,
                    critical,
                    max_seen: self.sensor_max[i],
                }
            })
            .collect();

        let val = self
            .sensors
            .get(self.selected_idx)
            .and_then(|s| s.celsius)
            .unwrap_or(0.0) as f64;

        if self.temp_history.len() >= HISTORY {
            self.temp_history.pop_front();
        }
        self.temp_history.push_back(val);
    }

    pub fn select_next(&mut self) {
        if !self.sensors.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.sensors.len();
            self.temp_history.clear(); 
        }
    }

    pub fn select_prev(&mut self) {
        if !self.sensors.is_empty() {
            if self.selected_idx == 0 {
                self.selected_idx = self.sensors.len() - 1;
            } else {
                self.selected_idx -= 1;
            }
            self.temp_history.clear();
        }
    }

    pub fn has_critical(&self) -> bool {
        self.sensors
            .iter()
            .any(|s| s.status() == ThermalStatus::Critical)
    }

    pub fn hottest(&self) -> Option<&SensorReading> {
        self.sensors
            .iter()
            .filter(|s| s.celsius.is_some())
            .max_by(|a, b| {
                a.celsius
                    .unwrap()
                    .partial_cmp(&b.celsius.unwrap())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}