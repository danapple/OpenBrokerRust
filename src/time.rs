use std::time::SystemTime;

pub fn current_time_millis() -> i64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).ok().unwrap().as_millis() as i64
}
