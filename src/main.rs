#![feature(slice_split_once)]

use std::{
    cmp::{max, min},
    collections::hash_map::Entry,
    env::args_os,
    fs::File,
    io::{stdout, Write as _},
    path::Path,
    sync::{Arc, Mutex},
};

use fixed::types::I48F16;
use fxhash::FxHashMap;
use memmap2::Mmap;
use mimalloc::MiMalloc;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator, FromParallelIterator},
    slice::{ParallelSlice, ParallelSliceMut},
};

use lazy_static::lazy_static;
use std::borrow::Cow;

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

#[inline(always)]
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

#[inline(always)]
/// experimental and cost more than fast_parse (hashing+lookup cost > than trick above)
fn lookup_temp(input: &[u8]) -> Value {
    //get values from hashmap instead of parsing them.
    //println!("getting value {:?}", std::str::from_utf8(input));
    let key = Cow::Borrowed(input);
    *TEMPURATURE_HASHMAP.get(&key).unwrap()
}

fn write_pair(city: &[u8], record: &Record, out: &mut Vec<u8>) {
    out.extend_from_slice(city);
    out.push(b'=');
    record.write(out);
}

//store possible temp values -100.9 => 100.9
lazy_static! {
    static ref TEMPURATURE_HASHMAP: FxHashMap<Cow<'static, [u8]>, Value> = {
        let mut hashmap = FxHashMap::with_capacity_and_hasher(100*10*2+10,Default::default());

        // Populate the hashmap with keys and values
        for i in (-1009..=1009).step_by(1) {
            let value = i as f64 / 10.0;
            let key_str = format!("{:.1}", value);
            let key_bytes = key_str.as_bytes().to_vec(); // Convert to owned Vec<u8>
            let key_cow: Cow<'static, [u8]> = Cow::Owned(key_bytes);
            hashmap.insert(key_cow, Value::from_num(value));
        }
        hashmap
    };
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
            || fxhash::FxHashMap::with_capacity_and_hasher(1000, Default::default()),
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
        .reduce(FxHashMap::default, |mut map1, map2| {
            map2.iter()
                .for_each(|(city, record2)| match map1.entry(city) {
                    Entry::Vacant(slot) => {
                        slot.insert(*record2);
                    }
                    Entry::Occupied(record1) => record1.into_mut().merge(record2),
                });

            map1
        });

    let mut sorted_data: Vec<(&&[u8], &Record)> = Vec::from_par_iter(data.par_iter());

    // Use Rayon to parallelize the sorting in chunks.
    // TODO : adjust the Chunk size depending on the target.
    let chunk_size: usize = 1_000_000_000/ num_cpus::get() ;
    sorted_data.par_chunks_mut(chunk_size).for_each(|chunk| {
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

    let out = Arc::new(Mutex::new(out));

    // Parallelize writing to output
    sorted_data.par_chunks(chunk_size).for_each(|chunk| {
        let mut local_out = Vec::with_capacity(chunk.len() * est_record_size);

        if let Some(&(city, record)) = chunk.first() {
            write_pair(city, record, &mut local_out);
            chunk.iter().skip(1).for_each(|&(city, record)| {
                local_out.extend_from_slice(b", ");
                write_pair(city, record, &mut local_out)
            });
        }

        // Extend the global output buffer in a synchronized way
        // Lock the mutex to safely extend the global output buffer
        let mut out = out.lock().unwrap();
        out.extend(local_out);
    });

    let mut out = out.lock().unwrap();
    out.extend_from_slice(b"}\n");

    stdout()
        .lock()
        .write_all(&out)
        .expect("failed to write to stdout");

    // No reason to waste time freeing memory and closing files and stuff
    std::process::exit(0);
}
