use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

const BUFFER_SIZE: usize = 1024 * 1024;

pub struct BitfieldArrayFile<const BITS: usize> {
    file: File,
    buffer: Vec<u8>,
    buffer_sub_ind: usize,
    curbyte: u8,
}

impl<const BITS: usize> BitfieldArrayFile<BITS> {
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        Self {
            file: OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(path)
                .unwrap(),
            buffer: Vec::new(),
            buffer_sub_ind: 0,
            curbyte: 0,
        }
    }

    pub fn push(&mut self, chunk: [bool; BITS]) {
        for &bit in &chunk {
            self.curbyte |= if bit { 1 } else { 0 } << self.buffer_sub_ind;
            self.buffer_sub_ind += 1;
            if self.buffer_sub_ind == 8 {
                self.buffer_sub_ind = 0;
                self.buffer.push(self.curbyte);
                self.curbyte = 0;
                if self.buffer.len() >= BUFFER_SIZE {
                    self.file.write_all(&self.buffer).unwrap();
                    self.buffer.clear();
                }
            }
        }
    }

    pub fn flush(&mut self) {
        self.file.write_all(&self.buffer).unwrap();
    }
}

pub fn get_bits<P: AsRef<Path>, const BITS: usize>(
    path: P,
    ind: usize,
) -> [bool; BITS] {
    let mut f = File::open(path).unwrap();

    let start_byte = (ind * BITS) / 8;
    let stop_byte = ((ind + 1) * BITS - 1) / 8;

    let mut buf = vec![0; stop_byte - start_byte + 1];

    f.seek(SeekFrom::Start(start_byte as u64)).unwrap();
    f.read(&mut buf).unwrap();

    let mut bitfield = [false; BITS];

    let start_bit = (ind * BITS) % 8;

    for i in 0..BITS {
        let bit_ind = start_bit + i;
        let byte_ind = bit_ind / 8;
        let sub_ind = bit_ind % 8;
        bitfield[i] = buf[byte_ind] & 1 << sub_ind != 0;
    }

    bitfield
}
