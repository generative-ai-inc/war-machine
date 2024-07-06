use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct MachineState {
    pub containers: HashMap<String, String>,
    pub ports: HashMap<String, i32>,
}
