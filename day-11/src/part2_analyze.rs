use miette::{IntoDiagnostic, Result, miette};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Element {
    value: usize,
}

impl Element {
    #[inline(always)]
    fn new(value: usize) -> Self {
        Self { value }
    }

    #[inline(always)]
    fn is_zero(&self) -> Result<bool> {
        Ok(self.value == 0)
    }

    #[inline(always)]
    fn is_even(&self) -> Result<bool> {
        if self.value == 0 {
            return Ok(false);
        }
        
        let mut num = self.value;
        let mut len = 0;
        while num > 0 {
            len += 1;
            num /= 10;
        }
        Ok(len % 2 == 0)
    }
    
    fn split_digits(&self) -> Result<Vec<Element>> {
        if self.value == 0 {
            return Ok(vec![Element::new(0)]);
        }

        let mut num = self.value;
        let mut len = 0;
        let mut power = 1;
        
        while num > 0 {
            len += 1;
            num /= 10;
        }
        
        for _ in 0..len/2 {
            power *= 10;
        }
        
        let right = self.value % power;
        let left = self.value / power;
        
        Ok(vec![Element::new(left), Element::new(right)])
    }
}

#[derive(Debug)]
struct SequenceStats {
    iteration: usize,
    length: usize,
    zeros: usize,
    evens: usize,
    odds: usize,
}

fn analyze_sequence(elements: &[Element]) -> Result<SequenceStats> {
    let mut zeros = 0;
    let mut evens = 0;
    let mut odds = 0;

    for element in elements {
        if element.is_zero()? {
            zeros += 1;
        } else if element.is_even()? {
            evens += 1;
        } else {
            odds += 1;
        }
    }

    Ok(SequenceStats {
        iteration: 0,
        length: elements.len(),
        zeros,
        evens,
        odds,
    })
}

pub fn process(input: &str, blink_count: usize) -> Result<String> {
    let mut current: Vec<Element> = input
        .split_whitespace()
        .map(|x| x.parse::<usize>().into_diagnostic())
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(Element::new)
        .collect();
        
    let mut next = Vec::with_capacity(current.len() * 2);
    let mut previous_stats = analyze_sequence(&current)?;
    
    println!("\nInitial state:");
    println!("Length: {}", previous_stats.length);
    println!("Zeros: {}", previous_stats.zeros);
    println!("Evens: {}", previous_stats.evens);
    println!("Odds: {}", previous_stats.odds);

    for iteration in 0..blink_count {
        next.clear();
        
        for element in &current {
            if element.is_zero()? {
                next.push(Element::new(1));
            } else if element.is_even()? {
                let split_elements = element.split_digits()?;
                next.extend(split_elements);
            } else {
                next.push(Element::new(element.value * 2024));
            }
        }
        
        let stats = analyze_sequence(&next)?;
        println!("\nIteration {}:", iteration + 1);
        println!("Length: {} (growth: {:.2}x)", stats.length, stats.length as f64 / previous_stats.length as f64);
        println!("Zeros: {}", stats.zeros);
        println!("Evens: {}", stats.evens);
        println!("Odds: {}", stats.odds);
        
        // Early exit if we detect exponential growth
        if stats.length > 1_000_000 {
            println!("\nSequence growing too large, analyzing pattern...");
            let growth_rate = stats.length as f64 / previous_stats.length as f64;
            println!("Growth rate per iteration: {:.2}x", growth_rate);
            
            // If we can predict the final length...
            let estimated_final_length = stats.length as f64 * growth_rate.powi((blink_count - iteration - 1) as i32);
            println!("Estimated final length: {:.2e}", estimated_final_length);
            
            return Ok(format!("Estimated length after {} iterations: {:.0}", blink_count, estimated_final_length));
        }
        
        previous_stats = stats;
        std::mem::swap(&mut current, &mut next);
    }

    Ok(current.len().to_string())
}