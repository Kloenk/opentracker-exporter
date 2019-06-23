
pub struct Config {
    /// verbosity level
    pub verbose: u8,

    /// address without http and without /stats?mode=everything
    pub url: String,

    /// port to listen on
    pub port: u16,
}

impl Config {
    /// new creates a new instance with default values
    pub fn new() -> Self {
        Self {
            verbose: 0,
            url: String::from("localhost"),
            port: 9999,
        }
    }

    /// execute application
    pub fn run(mut self) -> Result<(), String> {
        if self.verbose >= 1 {
            println!("Debug1: using opentracker stats on {}", self.url);
            println!("Debug1: listening for prometheus on port {}", self.port);
        }
        Ok(())
    }
}