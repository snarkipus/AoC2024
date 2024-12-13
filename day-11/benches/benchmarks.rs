use day_11::*;

fn main() {
    divan::main();
}

// Your existing benchmarks
#[divan::bench]
fn part1() {
    part1::process(divan::black_box(include_str!("../input1.txt",)), 25).unwrap();
}

#[divan::bench]
fn part1_claude() {
    part1_claude::process(divan::black_box(include_str!("../input1.txt",)), 25).unwrap();
}

// #[divan::bench]
// fn part2() {
//     part2::process(divan::black_box(include_str!("../input2.txt",))).unwrap();
// }

// New breakdown benchmarks
#[divan::bench]
fn get_digits() {
    let values = [1234usize, 100000, 202400];
    for &value in &values {
        let element = part1_claude::Element::new(value);
        divan::black_box(element.get_digits().unwrap());
    }
}

#[divan::bench]
fn split_digits() {
    let values = [1234usize, 100000, 202400];
    for &value in &values {
        let element = part1_claude::Element::new(value);
        divan::black_box(element.split_digits().unwrap());
    }
}

#[divan::bench]
fn element_operations() {
    let values = [12345usize, 1000000, 999999, 2024];
    for &value in &values {
        let element = part1_claude::Element::new(value);
        divan::black_box(element.get_digits().unwrap());
        if element.is_even().unwrap() {
            divan::black_box(element.split_digits().unwrap());
        }
        divan::black_box(element.is_zero().unwrap());
    }
}

#[divan::bench]
fn profile_operations() -> Vec<part1_claude::Element> {
    static mut TOTAL_OPS: usize = 0;
    
    let (ops, result) = {
        let mut operation_count = 0;
        let input_sequence = part1_claude::parse_input(include_str!("../input1.txt")).unwrap();
        let mut current = input_sequence.elements.clone();
        let mut next = Vec::with_capacity(current.len() * 2);

        for iteration in 0..25 {
            next.clear();
            let initial_len = current.len();
            
            for element in &current {
                operation_count += 1; // Count each element iteration
                
                if element.is_zero().unwrap() {
                    operation_count += 1;
                    next.push(part1_claude::Element::new(1));
                } else if element.is_even().unwrap() {
                    operation_count += 1;
                    let split_elements = element.split_digits().unwrap();
                    operation_count += 1;
                    next.extend(split_elements);
                } else {
                    operation_count += 1;
                    next.push(part1_claude::Element::new(element.value * 2024));
                }
            }
            
            eprintln!("Iteration {}: {} elements -> {} elements", iteration, initial_len, next.len());
            std::mem::swap(&mut current, &mut next);
        }

        (operation_count, current)
    };
    
    // Using unsafe to modify the static counter
    unsafe {
        TOTAL_OPS = ops;
        eprintln!("Total operations: {}", TOTAL_OPS);
    }
    
    result
}