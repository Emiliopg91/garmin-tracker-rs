pub mod session_parser;

use std::{fs::File, io::BufReader, path::Path};

use embedded_io_adapters::std::FromStd;
use rustyfit::{Decoder, StreamDecoder};

use self::errors::ParseFitFileError;

pub mod errors;

pub struct FitParser<'a> {
    path: &'a Path,
    stream: StreamDecoder<'a, FromStd<BufReader<File>>>,
}

impl<'a> FitParser<'a> {
    pub fn from_file<P>(path: &'a P, decoder: &'a mut Decoder) -> errors::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)
            .map_err(|e| ParseFitFileError::FileOpening(path_ref.display().to_string(), e))?;
        let reader = FromStd::new(BufReader::new(file));

        Ok(Self {
            path: path_ref,
            stream: decoder.stream(reader),
        })
    }

    #[cfg(debug_assertions)]
    pub fn debug_dump(mut self) -> Result<(), Box<dyn std::error::Error>> {
        use rustyfit::StreamingIterator;

        let dump_path = format!("{}.json", self.path.display());
        let mut entries = Vec::new();

        while let Some(event) = self.stream.next() {
            use rustyfit::DecoderEvent;

            let event = event.map_err(|e| {
                ParseFitFileError::FileReading(self.path.display().to_string(), Box::new(e))
            })?;

            if let DecoderEvent::Message(mesg) = event {
                entries.push(mesg.clone());
            }
        }

        let json = serde_json::to_string_pretty(&entries)?;
        std::fs::write(&dump_path, json)?;
        Ok(())
    }
}
