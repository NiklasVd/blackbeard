pub fn log(prefix: &str, text: &str) {
    println!("[{}] {}", prefix, text);
}

pub fn log_warning(text: &str) {
    log("W", text);
}

pub fn log_err(text: &str) {
    log("E", text);
}
