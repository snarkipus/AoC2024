use day_09::part1::{process, DiskState};
use day_09::part2::*;

fn main() {
    divan::main();
}

const SAMPLE_INPUT: &str = "2333133121414131402";
const REAL_INPUT: &str = include_str!("../input1.txt");

#[divan::bench]
fn part1() {
    day_09::part1::process(divan::black_box(REAL_INPUT.trim())).unwrap();
}

#[divan::bench]
fn part2() {
    day_09::part2::process(divan::black_box(REAL_INPUT.trim())).unwrap();
}

#[divan::bench]
fn part1_sample() {
    day_09::part1::process(divan::black_box(SAMPLE_INPUT)).unwrap();
}

// Size comparison benches
#[divan::bench]
fn parse_real() {
    DiskState::new(divan::black_box(REAL_INPUT.trim())).unwrap();
}

#[divan::bench]
fn parse_sample() {
    DiskState::new(divan::black_box(SAMPLE_INPUT)).unwrap();
}

#[divan::bench]
fn pack_real() {
    let mut state = DiskState::new(REAL_INPUT.trim()).unwrap();
    divan::black_box(&mut state).pack().unwrap();
}

#[divan::bench]
fn pack_sample() {
    let mut state = DiskState::new(SAMPLE_INPUT).unwrap();
    divan::black_box(&mut state).pack().unwrap();
}

// Print sizes for reference
#[divan::bench]
fn size_comparison() -> String {
    format!(
        "Sample input length: {}\nReal input length: {}\nFirst 50 chars: {:?}",
        SAMPLE_INPUT.len(),
        REAL_INPUT.len(),
        &REAL_INPUT[..50.min(REAL_INPUT.len())]
    )
}
