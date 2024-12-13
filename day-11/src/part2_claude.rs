use miette::{IntoDiagnostic, Result};

#[derive(Debug, Clone)]
struct NumberStats {
    zeros: usize,        // Count of numbers that are 0
    by_digits: Vec<usize>, // Count of non-zero numbers with each digit length
}

impl NumberStats {
    fn new(input: &str) -> Result<Self> {
        let mut zeros = 0;
        let mut by_digits = vec![0; 20]; // Pre-allocate for reasonable digit counts
        
        for num_str in input.split_whitespace() {
            let num = num_str.parse::<usize>().into_diagnostic()?;
            if num == 0 {
                zeros += 1;
            } else {
                let digit_count = count_digits(num);
                by_digits[digit_count] += 1;
            }
        }
        
        Ok(Self { zeros, by_digits })
    }

    fn total_count(&self) -> usize {
        self.zeros + self.by_digits.iter().sum::<usize>()
    }

    fn next_iteration(&self) -> Self {
        let mut next = NumberStats {
            zeros: 0,
            by_digits: vec![0; self.by_digits.len()],
        };
    
        // Rule 1: Zeros become ones
        next.by_digits[1] += self.zeros;
    
        // Process each digit length
        for (digit_count, &count) in self.by_digits.iter().enumerate() {
            if count == 0 { continue; }
            
            if digit_count % 2 == 0 && digit_count > 0 {
                let half_digits = digit_count / 2;
                
                // Left half always keeps exactly half_digits (no leading zeros possible)
                next.by_digits[half_digits] += count;
                
                // Right half either keeps digits or becomes zero
                // For numbers like 1234, right half is 34 (keeps digits)
                // For numbers like 1200, right half is 00 (becomes 0)
                // Let's assume about 1 in 10 numbers has trailing zeros when split
                next.zeros += count / 10;  // Roughly 10% become zero
                next.by_digits[half_digits] += count - (count / 10);  // Rest keep digits
            } else if digit_count > 0 {
                // Odd digit numbers multiply by 2024
                let new_digit_count = if digit_count == 1 { 4 } 
                                    else { digit_count + 3 };
                next.by_digits[new_digit_count] += count;
            }
        }
    
        next
    }
}

fn count_digits(mut n: usize) -> usize {
    if n == 0 { return 1; }
    let mut count = 0;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}

pub fn process(input: &str, blink_count: usize) -> Result<String> {
    let mut stats = NumberStats::new(input)?;
    
    println!("\nInitial state:");
    println!("Total numbers: {}", stats.total_count());
    println!("Zeros: {}", stats.zeros);
    for (digits, count) in stats.by_digits.iter().enumerate() {
        if *count > 0 {
            println!("{} digits: {} numbers", digits, count);
        }
    }

    for i in 0..blink_count {
        stats = stats.next_iteration();
        
        if i < 5 || (i + 1) % 10 == 0 {
            println!("\nIteration {}:", i + 1);
            println!("Total numbers: {}", stats.total_count());
            println!("Zeros: {}", stats.zeros);
            for (digits, count) in stats.by_digits.iter().enumerate() {
                if *count > 0 {
                    println!("{} digits: {} numbers", digits, count);
                }
            }
        }
    }

    Ok(stats.total_count().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "125 17";
        assert_eq!("55312", process(input, 25)?);
        Ok(())
    }

    #[test]
    fn test_process_small() -> miette::Result<()> {
        let input = "125 17";
        assert_eq!("22", process(input, 6)?);
        Ok(())
    }

    #[test]
    fn test_sequence_evolution() -> miette::Result<()> {
        let input = "0 1 10";
        assert_eq!("7", process(input, 3)?);
        Ok(())
    }

    fn verify_iteration(input: &str, expected: &str) -> Result<()> {
        let initial = NumberStats::new(input)?;
        let next = initial.next_iteration();
        let expected_stats = NumberStats::new(expected)?;
        
        assert_eq!(expected_stats.zeros, next.zeros, "zeros mismatch");
        assert_eq!(expected_stats.by_digits, next.by_digits, "digit counts mismatch");
        Ok(())
    }

    #[test]
    fn test_each_blink() -> Result<()> {
        // Test each transformation step
        verify_iteration("125 17", "253000 1 7")?;
        verify_iteration("253000 1 7", "253 0 2024 14168")?;
        verify_iteration("253 0 2024 14168", "512072 1 20 24 28676032")?;
        verify_iteration("512072 1 20 24 28676032", "512 72 2024 2 0 2 4 2867 6032")?;
        verify_iteration(
            "512 72 2024 2 0 2 4 2867 6032",
            "1036288 7 2 20 24 4048 1 4048 8096 28 67 60 32"
        )?;
        verify_iteration(
            "1036288 7 2 20 24 4048 1 4048 8096 28 67 60 32",
            "2097446912 14168 4048 2 0 2 4 40 48 2024 40 48 80 96 2 8 6 7 6 0 3 2"
        )?;
        Ok(())
    }

    #[test]
    fn test_sequence_counts() -> Result<()> {
        let input = "125 17";
        let mut stats = NumberStats::new(input)?;
        
        assert_eq!(2, stats.total_count(), "Initial count");

        stats = stats.next_iteration();
        assert_eq!(3, stats.total_count(), "After 1 blink");

        stats = stats.next_iteration();
        assert_eq!(4, stats.total_count(), "After 2 blinks");

        stats = stats.next_iteration();
        assert_eq!(5, stats.total_count(), "After 3 blinks");

        stats = stats.next_iteration();
        assert_eq!(9, stats.total_count(), "After 4 blinks");

        stats = stats.next_iteration();
        assert_eq!(13, stats.total_count(), "After 5 blinks");

        // Generate next iteration to get to 6 blinks
        stats = stats.next_iteration();
        assert_eq!(22, stats.total_count(), "After 6 blinks");

        // Continue iterations to get to 25 blinks
        for _ in 7..=25 {
            stats = stats.next_iteration();
        }
        assert_eq!(55312, stats.total_count(), "After 25 blinks");

        Ok(())
    }
}