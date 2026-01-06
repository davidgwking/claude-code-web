/// CLI application for the workspace
/// 
/// This binary demonstrates the usage of the core and utils libraries.

use core::Config;
use utils::{capitalize, greet};

fn main() {
    let config = Config::new("claude-code-web", "0.1.0");
    
    println!("=== Claude Code Web CLI ===");
    println!("{}", greet("Claude Code"));
    println!("Project: {}", config.name);
    println!("Version: {}", config.version);
    println!("Capitalized: {}", capitalize("rust workspace"));
}
