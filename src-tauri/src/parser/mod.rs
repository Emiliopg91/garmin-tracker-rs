pub mod session_parser;

use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use embedded_io_adapters::std::FromStd;
use ouroboros::self_referencing;
use rustyfit::{Decoder, StreamDecoder};

use self::errors::ParseFitFileError;

pub mod errors;

#[self_referencing]
pub struct FitParser {
    path: PathBuf,
    decoder: Decoder,

    #[borrows(mut decoder)]
    #[not_covariant]
    stream: StreamDecoder<'this, FromStd<BufReader<File>>>,
}

impl FitParser {
    pub fn from_file<P>(path: P) -> errors::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)
            .map_err(|e| ParseFitFileError::FileOpening(path_ref.display().to_string(), e))?;
        let reader = FromStd::new(BufReader::new(file));

        Ok(FitParserBuilder {
            path: path_ref.to_path_buf(),
            decoder: Decoder::new(),
            stream_builder: |decoder| decoder.stream(reader),
        }
        .build())
    }

    #[cfg(debug_assertions)]
    pub fn debug_dump(mut self) -> Result<(), Box<dyn std::error::Error>> {
        use rustyfit::{DecoderEvent, StreamingIterator};

        let path_string = self.borrow_path().display().to_string();
        let dump_path = format!("{path_string}.json");

        let entries = self.with_stream_mut(|stream| -> errors::Result<_> {
            let mut entries = Vec::new();
            while let Some(event) = stream.next() {
                let event = event.map_err(|e| {
                    ParseFitFileError::FileReading(path_string.clone(), Box::new(e))
                })?;

                if let DecoderEvent::Message(mesg) = event {
                    entries.push(mesg.clone());
                }
            }
            Ok(entries)
        })?;

        let json = serde_json::to_string_pretty(&entries)?;
        std::fs::write(&dump_path, json)?;
        Ok(())
    }
}
