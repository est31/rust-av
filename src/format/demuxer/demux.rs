#![allow(dead_code)]

use std::io::Error;
use data::packet::Packet;

pub trait Demuxer {
    fn open(&mut self);
    fn read_headers(&mut self) -> Result<(), Error>;
    fn read_packet(&mut self) -> Result<Packet, Error>;
}

pub struct DemuxerDescription {
    name: &'static str,
    description: &'static str,
    extensions: &'static [&'static str],
    mime: &'static [&'static str],
}

/// Least amount of data needed to check the bytestream structure
/// to match some known format.
pub const PROBE_DATA: usize = 4 * 1024;

/// Probe threshold values
pub enum Score {
    /// Minimum acceptable value, a file matched just by the extension
    EXTENSION = 50,
    /// The underlying layer provides the information, trust it up to a point
    MIME = 75,
    /// The data actually match a format structure
    MAX = 100,
}

pub trait DemuxerBuilder {
    fn describe(&self) -> &'static DemuxerDescription;
    fn probe(&self, data: &[u8; PROBE_DATA]) -> u8;
    fn alloc(&self) -> Box<Demuxer>;
}

pub fn probe<'a>(demuxers: &[&'static DemuxerBuilder],
                 data: &[u8; PROBE_DATA])
                 -> Option<&'a DemuxerBuilder> {
    let mut max = u8::min_value();
    let mut candidate: Option<&DemuxerBuilder> = None;
    for builder in demuxers {
        let score = builder.probe(data);

        if score > max {
            max = score;
            candidate = Some(*builder);
        }
    }

    if max > Score::EXTENSION as u8 {
        candidate
    } else {
        None
    }
}

macro_rules! module {
    {
        ($name:ident) {
            open => $open:block
            read_headers => $read_headers:block
            read_packet => $read_packet:block

            describe => $describe:block
            probe => $probe:block
            alloc => $alloc:block
        }
    } => {
        interpolate_idents! {
            struct [$name Demuxer];
            struct [$name DemuxerBuilder];

            impl Demuxer for [$name Demuxer] {
                fn open(&mut self) $open
                fn read_headers(&mut self) -> Result<(), Error> $read_headers
                fn read_packet(&mut self) -> Result<Packet, Error> $read_packet
            }

            impl DemuxerBuilder for [$name DemuxerBuilder] {
                fn describe(&self) -> &'static DemuxerDescription $describe
                fn probe(&self, data: &[u8; PROBE_DATA]) -> u8 $probe

                fn alloc(&self) -> -> Box<Demuxer> $alloc
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::Error;
    use data::packet::Packet;

    module! {
        (Test) {
            open => { () }
            read_headers => { Ok(()) }
            read_packet => { unimplemented!() }

            describe => {
                const D: &'static DemuxerDescription = &DemuxerDescription {
                    name: "Test",
                    description: "Test demuxer",
                    extensions: &["test", "t"],
                    mime: &["x-application/test"],
                };

                D
            }

            probe => {
                if data[0] == 0 {
                    Score::MAX as u8
                } else {
                    0
                }
            }

            alloc => {
                let demux = TestDemuxer {};

                box demux
            }
        }
    }

    const DEMUXER_BUILDERS: [&'static DemuxerBuilder; 1] = [&TestDemuxerBuilder {}];

    #[test]
    fn probe_demuxer() {
        let mut buf = [1; PROBE_DATA];

        match probe(&DEMUXER_BUILDERS, &buf) {
            Some(_) => panic!(),
            None => (),
        };

        buf[0] = 0;

        match probe(&DEMUXER_BUILDERS, &buf) {
            Some(_) => (),
            None => panic!(),
        };
    }
}
