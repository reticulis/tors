use phf::phf_map;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Achievements {
    FirstTask,
    MissionComplete,
}
// pub static ACHIEVEMENTS_LVL: phf::Map<&'static str, Achievements> = phf_map! {
//     "5" => MissionComplete,
// };
