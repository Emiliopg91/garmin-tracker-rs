use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct HeartRateData {
    pub value: u8,
    pub zone: u8,
}

impl From<u8> for HeartRateData {
    fn from(value: u8) -> Self {
        let rate = (value as f64) / 189_f64;
        let zone = if rate < 0.6 {
            1
        } else {
            if rate < 0.7 {
                2
            } else {
                if rate < 0.8 {
                    3
                } else {
                    if rate < 0.9 { 4 } else { 5 }
                }
            }
        };

        Self { value, zone }
    }
}
