use rusqlite_orm_macros::Entity;

#[derive(Clone, Entity)]
#[entity("heart_rate")]
#[primary_key(session)]
pub struct HeartRate {
    pub session: i64,
    pub records: Vec<u8>,
}

impl HeartRate {
    pub fn get_time_in_zones(&self, total_elapsed_time: f64) -> Vec<i32> {
        let time_fraction = total_elapsed_time / (self.records.len() as f64);
        let max_hr = 189_u8.max(*self.records.iter().max().unwrap()) as f64;

        let mut zone_counts = [0_u32; 5];
        for &hr in &self.records {
            let rate = hr as f64 / max_hr;
            let zone = if rate < 0.6 {
                0
            } else if rate < 0.7 {
                1
            } else if rate < 0.8 {
                2
            } else if rate < 0.9 {
                3
            } else {
                4
            };
            zone_counts[zone] += 1;
        }

        let mut int_times = zone_counts
            .iter()
            .map(|&count| (time_fraction * count as f64).round() as i64)
            .collect::<Vec<_>>();
        let acc: i64 = int_times.iter().sum();
        let total_secs = total_elapsed_time.round() as i64;
        let diff = total_secs - acc;
        int_times[0] += diff;

        int_times.into_iter().map(|v| v as i32).collect()
    }
}
