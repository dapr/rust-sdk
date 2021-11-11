use serde::Serialize;
use std::time::Duration;

#[derive(Debug)]
pub struct DaprDuration {
    duration: Duration,
}

impl DaprDuration {
    pub fn from(duration: Duration) -> Self {
        Self { duration }
    }
}

impl Serialize for DaprDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // write in format expected by Dapr, it only accepts h, m, s, ms, us(micro), ns
        let dapr_time_str = match self.duration.as_millis() {
            0 => "0s".to_string(),
            millis => {
                const ONE_HOUR: u128 = 1000 * 3600;
                const ONE_MIN: u128 = 1000 * 60;
                const ONE_SEC: u128 = 1000;
                let hours = millis / ONE_HOUR;
                let mins = (millis % ONE_HOUR) / ONE_MIN;
                let seconds = (millis % ONE_MIN) / ONE_SEC;
                let millis = millis % ONE_SEC;
                
                format!("{}h {}m {}s {}ms", hours, mins, seconds, millis)
            }
        };

        serializer.serialize_str(&dapr_time_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn works_with_content() {

        let hours = 45;
        let minutes = 32;
        let seconds = 23;
        let millis = 234;
        let duration = DaprDuration::from(
            std::time::Duration::from_secs_f64(hours as f64 *3600f64 + minutes as f64 * 60f64 + seconds as f64 + millis as f64 / 1000.0));
        let expected = format!("\"{}h {}m {}s {}ms\"", hours, minutes, seconds, millis);
        assert_eq!(expected, serde_json::to_string(&duration).unwrap());
    }

    #[test]
    fn works_with_empty_duration() {

        let duration = DaprDuration::from(
            std::time::Duration::from_secs(0));
        let expected = format!("\"0s\"");
        assert_eq!(expected, serde_json::to_string(&duration).unwrap());
    }
}
