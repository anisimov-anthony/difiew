use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp_millis() -> Option<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|dur| dur.as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_some_millis() {
        let ts = timestamp_millis();
        assert!(ts.is_some());
        let millis = ts.unwrap();
        assert!(millis > 1_600_000_000_000, "timestamp should be after 2020");
        assert!(
            millis < 2_000_000_000_000,
            "timestamp should be before ~2033"
        );
    }

    #[test]
    fn returns_increasing_values() {
        let ts1 = timestamp_millis().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let ts2 = timestamp_millis().unwrap();
        assert!(ts2 > ts1);
    }
}
