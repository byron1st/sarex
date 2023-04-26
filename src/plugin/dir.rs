use std::path::PathBuf;

const SAREX_DIR: &str = ".sarex";
const PLUGIN_DIR: &str = "plugins";

pub fn get_plugin_dir() -> PathBuf {
    let mut p = PathBuf::new();
    if let Some(home) = dirs::home_dir() {
        p.push(home);
        p.push(SAREX_DIR);
        p.push(PLUGIN_DIR);
    }

    p
}
