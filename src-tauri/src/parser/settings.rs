use crate::parser::{
    FitParser,
    errors::{self, ParseFitFileError},
};
use rustyfit::{
    DecoderEvent, StreamingIterator,
    profile::{mesgdef, typedef::MesgNum},
};

#[derive(Clone, Debug)]
pub struct DeviceSettings {
    pub max_heart_rate: u8,
}

impl Default for DeviceSettings {
    fn default() -> Self {
        Self {
            max_heart_rate: 189,
        }
    }
}

impl TryFrom<FitParser> for DeviceSettings {
    type Error = errors::ParseFitFileError;
    fn try_from(mut value: FitParser) -> Result<Self, Self::Error> {
        let mut max_heart_rate = None;
        let path_string = value.borrow_path().display().to_string();

        value.with_stream_mut(|stream| -> errors::Result<()> {
            let mut zt_parsed = false;
            while let Some(event) = stream.next() {
                let event = event.map_err(|e| {
                    ParseFitFileError::FileReading(path_string.clone(), Box::new(e))
                })?;
                if let DecoderEvent::Message(msg) = event
                    && msg.num == MesgNum::ZONES_TARGET
                {
                    if !zt_parsed {
                        let dev_set = mesgdef::ZonesTarget::from(msg);
                        max_heart_rate = Some(dev_set.max_heart_rate);
                        zt_parsed = true;
                    }
                }
            }

            Ok(())
        })?;

        Ok(DeviceSettings {
            max_heart_rate: max_heart_rate.unwrap_or(189),
        })
    }
}
