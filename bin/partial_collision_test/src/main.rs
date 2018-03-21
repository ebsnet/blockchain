extern crate bincode;
#[macro_use]
extern crate serde_derive;

extern crate sha2;

use bincode::serialize;

use sha2::{Digest, Sha256};

use std::io::{BufWriter, Write};
use std::fs::File;

const DIFF: usize = 16;

#[derive(Serialize)]
struct Data {
    data: String,
    nonce: u64,
}

impl Data {
    fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    fn as_bytes(&self) -> Vec<u8> {
        serialize(self, bincode::Infinite).unwrap()
    }

    fn check(&self, diff: usize) -> bool {
        let bytes = self.as_bytes();
        let hash = Sha256::digest(&bytes);
        hash.iter()
            .take((diff / 8) + 1)
            .fold((diff, true), |(d, b), byte| {
                let leading_zeros = byte.leading_zeros();
                if d >= 8 {
                    (d - 8, b && leading_zeros == 8)
                } else {
                    (d, b && leading_zeros >= d as u32)
                }
            })
            .1
    }
}

fn main() {
    let mut args = std::env::args();
    let progname = args.next().unwrap();
    if let (Some(inp), Some(out)) = (args.next(), args.next()) {
        let mut data = Data {
            data: inp,
            nonce: 0,
        };
        while !data.check(DIFF) {
            data.increment_nonce();
            // println!("nonce: {}", data.nonce);
        }
        println!("nonce: {}", data.nonce);
        println!("hash: {:?}", Sha256::digest(&data.as_bytes()));
        let mut write = BufWriter::new(File::create(out).unwrap());
        write.write_all(&data.as_bytes()).unwrap();
    } else {
        writeln!(std::io::stderr(), "Usage: {} data output", progname).ok();
        std::process::exit(1);
    }
}
