use crate::position::{Position, ShiftDirection};
use crate::random::RandomDistConfig;
use log::warn;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(RustEmbed)]
#[folder = "data/gen_configs/"]
pub struct GenerationConfigStorage;

#[derive(RustEmbed)]
#[folder = "data/map_configs/"]
pub struct MapConfigStorage;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MapConfig {
    /// name of the map config
    pub name: String,

    /// shape of a map using waypoints
    pub waypoints: Vec<Position>,

    /// width of the map
    pub width: usize,

    /// height of the map
    pub height: usize,
}

impl MapConfig {
    pub fn get_all_configs() -> HashMap<String, MapConfig> {
        let mut configs = HashMap::new();

        for file_name in MapConfigStorage::iter() {
            let file = MapConfigStorage::get(&file_name).unwrap();
            let data = std::str::from_utf8(&file.data).unwrap();
            let config: MapConfig = serde_json::from_str(data).unwrap();
            configs.insert(config.name.clone(), config);
        }

        configs
    }

    pub fn save(&self, path: &str) {
        let mut file = File::create(path).expect("failed to create config file");
        let serialized = serde_json::to_string_pretty(self).expect("failed to serialize config");
        file.write_all(serialized.as_bytes())
            .expect("failed to write to config file");
    }

    /// This function defines the initial default config for actual map generator
    pub fn get_initial_config() -> MapConfig {
        let file = MapConfigStorage::get("small_s.json").unwrap();
        let data = std::str::from_utf8(&file.data).unwrap();
        let config: MapConfig = serde_json::from_str(data).unwrap();
        config
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(default)]
pub struct GenerationConfig {
    /// name of the preset
    pub name: String,

    /// this can contain any description of the generation preset
    pub description: Option<String>,

    /// stores the GenerationConfig version for future migration
    pub version: String,

    /// probability for mutating inner radius
    pub inner_rad_mut_prob: f32,

    /// probability for mutating inner size
    pub inner_size_mut_prob: f32,

    /// probability for mutating outer radius
    pub outer_rad_mut_prob: f32,

    /// probability for mutating outer size
    pub outer_size_mut_prob: f32,

    /// probability weighting for random selection from best to worst towards next goal
    pub shift_weights: RandomDistConfig<ShiftDirection>,

    // ===================================[ platforms ]==========================================
    /// min distance between platforms
    pub plat_min_distance: usize,
    pub plat_width_bounds: (usize, usize),
    pub plat_height_bounds: (usize, usize),
    pub plat_min_empty_height: usize,

    /// allow "soft" overlaps -> non-empty blocks below platform (e.g. freeze)
    pub plat_soft_overhang: bool,

    // ===================================[ ]==========================================
    /// probability for doing the last shift direction again
    pub momentum_prob: f32,

    /// maximum distance from empty blocks to nearest non empty block for obstacle generation
    /// TODO: rename in new version bump, as this is not self explanatory at all xd
    pub max_distance: f32,

    /// min distance to next waypoint that is considered reached
    pub waypoint_reached_dist: usize,

    /// probabilities for (inner_kernel_size, probability)
    pub inner_size_probs: RandomDistConfig<usize>,

    /// probabilities for (outer_kernel_margin, probability)
    pub outer_margin_probs: RandomDistConfig<usize>,

    /// probabilities for (kernel circularity, probability)
    pub circ_probs: RandomDistConfig<f32>,

    /// (min, max) distance for skips
    pub skip_length_bounds: (usize, usize),

    /// min distance between skips. If a skip is validated, all neighbouring skips closer than this
    /// range are invalidated.
    pub skip_min_spacing_sqr: usize,

    /// maximum amount of the level is allowed to skip. This ensures that different parts of a map
    /// are not connected.
    pub max_level_skip: usize,

    /// min unconnected freeze obstacle size
    pub min_freeze_size: usize,

    /// enable pulse
    pub enable_pulse: bool,

    /// TODO:
    pub pulse_straight_delay: usize,
    pub pulse_corner_delay: usize,
    pub pulse_max_kernel_size: usize,

    /// number of initial walker steps to perform fading. Will fade from max to min kernel size.
    pub fade_steps: usize,

    /// initial max kernel size for fading
    pub fade_max_size: usize,

    /// goal min kernel size for fading
    pub fade_min_size: usize,

    /// maximum valid distance between subwaypoints
    pub max_subwaypoint_dist: f32,

    /// maximum distance that subwaypoints are shifted from their base position
    pub subwaypoint_max_shift_dist: f32,

    /// how close previous positions may be locked
    pub pos_lock_max_dist: f32,

    /// how many steps the locking may lack behind until the generation is considered "stuck"
    pub pos_lock_max_delay: usize,

    /// size of area that is locked
    pub lock_kernel_size: usize,
}

impl GenerationConfig {
    /// returns an error if the configuration would result in a crash
    pub fn validate(&self) -> Result<(), &'static str> {
        // 1. Check that there is no inner kernel size of 0
        for inner_size in self.inner_size_probs.values.as_ref().unwrap().iter() {
            if *inner_size == 0 {
                return Err("Invalid Config! (inner_size = 0)");
            }
        }

        // 2. Check fade config
        if self.fade_max_size == 0 || self.fade_min_size == 0 {
            return Err("fade kernel sizes must be larger than zero");
        }

        // 3. Check subwaypoint config
        if self.max_subwaypoint_dist <= 0.0 {
            return Err("max subwaypoint distance must be >0");
        }

        Ok(())
    }

    pub fn save(&self, path: &str) {
        let mut file = File::create(path).expect("failed to create config file");
        let serialized = serde_json::to_string_pretty(self).expect("failed to serialize config");
        file.write_all(serialized.as_bytes())
            .expect("failed to write to config file");
    }

    pub fn load(path: &str) -> GenerationConfig {
        let serialized_from_file = fs::read_to_string(path).expect("failed to read config file");
        let deserialized: GenerationConfig =
            serde_json::from_str(&serialized_from_file).expect("failed to deserialize config file");

        deserialized
    }

    pub fn get_all_configs() -> HashMap<String, GenerationConfig> {
        let mut configs = HashMap::new();

        for file_name in GenerationConfigStorage::iter() {
            let file = GenerationConfigStorage::get(&file_name).unwrap();
            let data = std::str::from_utf8(&file.data).unwrap();
            match serde_json::from_str::<GenerationConfig>(data) {
                Ok(config) => {
                    configs.insert(config.name.clone(), config);
                }
                Err(e) => {
                    warn!("couldn't parse gen config {}: {}", file_name, e);
                }
            }
        }

        configs
    }

    /// This function defines the initial default config for actual map generator
    pub fn get_initial_gen_config() -> GenerationConfig {
        if let Some(file) = GenerationConfigStorage::get("hardV2.json") {
            if let Ok(data) = std::str::from_utf8(&file.data) {
                if let Ok(config) = serde_json::from_str(data) {
                    return config;
                }
            }
        }

        return GenerationConfig::default();
    }
}

impl Default for GenerationConfig {
    /// Default trait should mainly be used to get default values for individual arguments
    /// instead of being used as an actual generation config. (use get_initial_config())
    fn default() -> GenerationConfig {
        GenerationConfig {
            name: "default".to_string(),
            description: None,
            version: "1.0".to_string(),
            inner_rad_mut_prob: 0.25,
            inner_size_mut_prob: 0.5,
            outer_rad_mut_prob: 0.25,
            outer_size_mut_prob: 0.5,
            shift_weights: RandomDistConfig::new(None, vec![0.4, 0.22, 0.2, 0.18]),
            plat_min_distance: 75,
            plat_width_bounds: (3, 5),
            plat_height_bounds: (1, 2),
            plat_min_empty_height: 4,
            plat_soft_overhang: false,
            momentum_prob: 0.01,
            max_distance: 3.0,
            waypoint_reached_dist: 250,
            inner_size_probs: RandomDistConfig::new(Some(vec![3, 5]), vec![0.25, 0.75]),
            outer_margin_probs: RandomDistConfig::new(Some(vec![0, 2]), vec![0.5, 0.5]),
            circ_probs: RandomDistConfig::new(Some(vec![0.0, 0.6, 0.8]), vec![0.75, 0.15, 0.05]),
            skip_min_spacing_sqr: 45,
            skip_length_bounds: (3, 11),
            max_level_skip: 90,
            min_freeze_size: 0,
            enable_pulse: false,
            pulse_corner_delay: 5,
            pulse_straight_delay: 10,
            pulse_max_kernel_size: 4,
            fade_steps: 60,
            fade_max_size: 6,
            fade_min_size: 3,
            max_subwaypoint_dist: 50.0,
            subwaypoint_max_shift_dist: 5.0,
            pos_lock_max_delay: 1000,
            pos_lock_max_dist: 20.0,
            lock_kernel_size: 9,
        }
    }
}

impl Default for MapConfig {
    fn default() -> MapConfig {
        MapConfig {
            name: "default".to_string(),
            waypoints: vec![
                Position::new(50, 250),
                Position::new(250, 250),
                Position::new(250, 150),
                Position::new(50, 150),
                Position::new(50, 50),
                Position::new(250, 50),
            ],
            width: 300,
            height: 300,
        }
    }
}
