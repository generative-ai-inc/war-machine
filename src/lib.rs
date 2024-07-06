pub mod lib {
    pub mod commands;
    pub mod config;
    pub mod secrets;
    pub mod system;
    pub mod utils;
}

pub mod models {
    pub mod config;
}

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
