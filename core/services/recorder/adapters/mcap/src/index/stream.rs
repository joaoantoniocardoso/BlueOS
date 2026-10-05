//! Little-endian reads for MCAP record payloads.

pub(crate) struct EndOfFile;

pub(crate) struct ReadDataStream<'a, R: std::io::Read> {
    pub recording: &'a mut R,
    pub count: usize,
}

impl<'a, R: std::io::Read> ReadDataStream<'a, R> {
    pub(super) fn read(&mut self, length: usize) -> Result<Vec<u8>, EndOfFile> {
        if length == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![0_u8; length];
        let read = match self.recording.read(&mut buffer) {
            Ok(value) => value,
            Err(_) => return Err(EndOfFile),
        };
        self.count += read;
        if read == 0 || read < length {
            Err(EndOfFile)
        } else {
            Ok(buffer)
        }
    }

    pub(super) fn read2(&mut self) -> Result<u16, EndOfFile> {
        let bytes = self.read(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub(super) fn read4(&mut self) -> Result<u32, EndOfFile> {
        let bytes = self.read(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(super) fn read8(&mut self) -> Result<u64, EndOfFile> {
        let bytes = self.read(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}
