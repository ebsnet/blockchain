use std::cmp;
use std::collections::BTreeSet;
use std::fmt;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use chrono::naive::NaiveDateTime;

use cryptography::PublicKey;

/// Representation of a invoice. Contains the user to be billed and a BTreeSet of all invoice
/// positions. The BTreeSet was chosen over any other collection because it is sorted by default.
pub struct Invoice {
    user: PublicKey,
    positions: BTreeSet<InvoicePosition>,
}

impl Invoice {
    pub fn new(user: PublicKey, positions: BTreeSet<InvoicePosition>) -> Self {
        Self { user, positions }
    }

    pub fn write_to_file<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let mut writer = BufWriter::new(File::create(path)?);
        write!(writer, "{}", self)
    }

    pub fn user(&self) -> &PublicKey {
        &self.user
    }
}

impl fmt::Display for Invoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Invoice for {}", self.user)?;
        self.positions
            .iter()
            .fold(Ok(()), |acc, pos| acc.and(writeln!(f, "\t{}", pos)))
    }
}

pub struct InvoicePosition {
    date: NaiveDateTime,
    usage: u64,
}

impl InvoicePosition {
    pub fn new(date: u64, usage: u64) -> Self {
        Self {
            date: NaiveDateTime::from_timestamp(date as i64, 0),
            usage,
        }
    }
}

impl fmt::Display for InvoicePosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.date, self.usage)
    }
}

impl PartialEq for InvoicePosition {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
    }
}

impl Eq for InvoicePosition {}

impl PartialOrd for InvoicePosition {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

impl Ord for InvoicePosition {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.date.cmp(&other.date)
    }
}
