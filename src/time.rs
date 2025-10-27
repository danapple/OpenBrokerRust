use std::time::SystemTime;

pub fn current_time_millis() -> i64 {
    let system_time_result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    match system_time_result {
        Ok(system_time) => system_time.as_millis() as i64,
        Err(time_error) => panic!("Unable to get system time: {}", time_error.to_string()),
    }
}
