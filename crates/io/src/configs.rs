use ahash::HashMap;
use mavelin_shared::Color;
use mavelin_world::Biome;
use serde::{Deserialize, Serialize};

// base_foliage_color = "#FFFFFF"
// base_sky_color = ""
// base_sky_color = ""
//
// [biomes.biome_name]
//

#[derive(Debug, Deserialize, Serialize)]
pub struct BiomeColorConfig {
    pub foliage_color: Option<Color>,
    pub water_color: Option<Color>,
    pub sky_color: Option<Color>,
    pub fog_color: Option<Color>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ColorConfig {
    pub base_foliage_color: Color,
    pub base_water_color: Color,
    pub base_sky_color: Color,
    pub base_fog_color: Option<Color>,
    pub biomes: HashMap<Biome, BiomeColorConfig>,
}

impl ColorConfig {
    #[allow(clippy::missing_errors_doc)]
    pub fn from_toml_slice(data: &[u8]) -> Result<Self, toml::de::Error> {
        toml::from_slice(data)
    }
}
