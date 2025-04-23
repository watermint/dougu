pub trait CommandLayer {
    fn handle_command(&self, command: &str, args: Vec<String>) -> Result<(), String>;
    fn is_empty(&self, result: &str) -> bool;
} 