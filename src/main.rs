#![feature(slice_split_once)]

use std::{
    cmp::{max, min},
    collections::hash_map::Entry,
    env::args_os,
    fs::File,
    io::{stdout, Write as _},
    path::Path,
};

use ahash::HashMapExt;
use fixed::types::I48F16;
use memmap2::Mmap;
use mimalloc::MiMalloc;
use rayon::{
    iter::ParallelIterator,
    slice::{ParallelSlice, ParallelSliceMut},
};
use rayon::iter::IntoParallelRefIterator;

type Value = I48F16;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Record {
    min: Value,
    sum: Value,
    max: Value,
    count: usize,
}

impl Record {
    fn merge(&mut self, other: &Self) {
        self.min = min(self.min, other.min);
        self.max = max(self.max, other.max);
        self.sum = self.sum.wrapping_add(other.sum);
        self.count = self.count.wrapping_add(other.count);
    }

    fn add(&mut self, value: Value) {
        self.min = min(self.min, value);
        self.max = max(self.max, value);
        self.sum = self.sum.wrapping_add(value);
        self.count = self.count.wrapping_add(1);
    }

    fn new(value: Value) -> Self {
        Self {
            min: value,
            max: value,
            sum: value,
            count: 1,
        }
    }

    fn avg(&self) -> Value {
        self.sum / Value::from_num(self.count)
    }

    fn write(&self, out: &mut Vec<u8>) {
        let min = self.min;
        let max = self.max;
        let avg = self.avg();

        write!(out, "{min:.1}/{avg:.1}/{max:.1}").expect("write to vec is infallbile")
    }
}

fn fast_parse(input: &[u8]) -> Value {
    let neg = input[0] == b'-';
    let len = input.len();

    let (d1, d2, d3) = match (neg, len) {
        (false, 3) => (0, input[0] - b'0', input[2] - b'0'),
        (false, 4) => (input[0] - b'0', input[1] - b'0', input[3] - b'0'),
        (true, 4) => (0, input[1] - b'0', input[3] - b'0'),
        (true, 5) => (input[1] - b'0', input[2] - b'0', input[4] - b'0'),
        _ => unreachable!(),
    };

    let int = (d1 as i16 * 100) + (d2 as i16 * 10) + (d3 as i16);
    let int = if neg { -int } else { int };

    Value::from_num(int) / Value::from_num(10)
}

fn write_pair(city: &[u8], record: &Record, out: &mut Vec<u8>) {
    out.extend_from_slice(city);
    out.push(b'=');
    record.write(out);
}

fn main() {
    // Simple mega parallel rayon solution
    let path = args_os()
        .nth(1)
        .expect("provide a path to the file as an argument");

    let path = Path::new(&path);
    let file = File::open(path).expect("failed to open file");
    let mapped_data = unsafe { Mmap::map(&file) }.expect("failed to create memory map");

    let raw_data = &*mapped_data;

    let raw_data = raw_data.strip_suffix(b"\n").unwrap_or(raw_data);

    let data = raw_data
        .par_split(|&b| b == b'\n')
        .map(|row| {
            let (city, sample) = row.split_once(|&b| b == b';').expect("no ; separator");
            let sample: Value = fast_parse(sample);
            (city, sample)
        })
        .fold(
            || ahash::HashMap::with_capacity(1000),
            |mut map, (city, sample)| {
                match map.entry(city) {
                    Entry::Vacant(slot) => {
                        slot.insert(Record::new(sample));
                    }
                    Entry::Occupied(record) => record.into_mut().add(sample),
                }
                map
            },
        )
        .reduce(ahash::HashMap::new, |mut map1, map2| {
            map2.iter()
                .for_each(|(city, record2)| match map1.entry(city) {
                    Entry::Vacant(slot) => {
                        slot.insert(*record2);
                    }
                    Entry::Occupied(record1) => record1.into_mut().merge(record2),
                });

            map1
        });

    let mut sorted_data: Vec<(&[u8], &Record)> =
        data.par_iter().map(|(&city, record)| (city, record)).collect();

    // Use Rayon to parallelize the sorting in chunks.
    const CHUNK_SIZE: usize = 1000000; 
    sorted_data.par_chunks_mut(CHUNK_SIZE).for_each(|chunk| {
        chunk.par_sort_unstable_by_key(|&(city, _)| city);
    });

    // Merge the sorted chunks.
    sorted_data.sort_unstable_by_key(|&(city, _)| city);

    let est_record_size =
        20 // city name
        + 1 // eq
        + (3 * 4) // values
        + 2 // slashes
        + 2 // comma-space
    ;

    let mut out: Vec<u8> = Vec::with_capacity(sorted_data.len() * est_record_size);

    out.push(b'{');

    let mut sorted_data_iter = sorted_data.iter();

    if let Some(&(city, record)) = sorted_data_iter.next() {
        write_pair(city, record, &mut out);
        sorted_data_iter.for_each(|&(city, record)| {
            out.extend_from_slice(b", ");
            write_pair(city, record, &mut out)
        });
    }

    out.extend_from_slice(b"}\n");

    stdout()
        .lock()
        .write_all(&out)
        .expect("failed to write to stdout");

    // No reason to waste time freeing memory and closing files and stuff
    std::process::exit(0);
}