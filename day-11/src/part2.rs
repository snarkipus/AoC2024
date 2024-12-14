use itertools::Itertools;
use miette::{miette, IntoDiagnostic, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Element {
    value: usize,
}

impl Element {
    #[inline(always)]
    fn new(value: usize) -> Self {
        Self { value }
    }

    fn get_digits(&self) -> Result<(Vec<usize>, usize)> {
        if self.value == 0 {
            return Ok((vec![0], 0));
        }

        let mut num = self.value;
        let mut digits = Vec::new();

        while num > 0 {
            digits.push(num % 10);
            num /= 10;
        }

        digits.reverse();
        let length = digits.len();
        Ok((digits, length))
    }

    fn is_zero(&self) -> Result<bool> {
        Ok(self.value == 0)
    }

    fn _flip_zero(&mut self) -> Result<()> {
        if self.value == 0 {
            self.value = 1;
        } else {
            return Err(miette!("Attempted to flip non-zero value"));
        }
        Ok(())
    }

    fn is_even(&self) -> Result<bool> {
        let (_, length) = self.get_digits()?;
        Ok(length % 2 == 0)
    }

    fn split_digits(&self) -> Result<Vec<Element>> {
        // Optimized implementation without string conversion
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

        for _ in 0..len / 2 {
            power *= 10;
        }

        let right = self.value % power;
        let left = self.value / power;

        Ok(vec![Element::new(left), Element::new(right)])
    }

    fn _mult_2024(&mut self) -> Result<()> {
        let value = self.value * 2024;
        self.value = value;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sequence {
    elements: Vec<Element>,
}

#[tracing::instrument]
pub fn process(input: &str, blink_count: usize) -> Result<String> {
    let sequence = parse_input(input)?;

    // Use iterative processing to avoid stack overflow
    let final_elements = process_sequence_iterative(&sequence, blink_count)?;

    Ok(final_elements.len().to_string())
}

fn parse_input(input: &str) -> Result<Sequence> {
    let elements: Vec<Element> = input
        .split_whitespace()
        .map(|x| x.parse::<usize>().into_diagnostic())
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(Element::new)
        .collect();
    Ok(Sequence { elements })
}

// This version is kept for test compatibility
fn process_sequence(input_sequence: &Sequence, count: usize) -> Result<Vec<Element>> {
    process_sequence_iterative(input_sequence, count)
}

fn process_sequence_iterative(input_sequence: &Sequence, count: usize) -> Result<Vec<Element>> {
    if count == 0 {
        return Ok(input_sequence.elements.clone());
    }

    // Use two buffers to avoid allocations
    let mut current = input_sequence.elements.clone();
    let mut next = Vec::with_capacity(current.len() * 2);

    for _ in 0..count {
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

        // Ensure next buffer has enough capacity for next iteration
        if next.len() > current.capacity() {
            current = Vec::with_capacity(next.len() * 2);
        }

        // Swap buffers
        std::mem::swap(&mut current, &mut next);
    }

    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};
    use test_log;

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

    #[test_log::test]
    fn test_single_process_sequence() -> miette::Result<()> {
        let input = "0 1 10 99 999";
        let sequence = parse_input(input)?;
        let expected = vec![
            Element::new(1),
            Element::new(2024),
            Element::new(1),
            Element::new(0),
            Element::new(9),
            Element::new(9),
            Element::new(2021976),
        ];
        assert_eq!(expected, process_sequence(&sequence, 1)?);
        Ok(())
    }

    #[fixture]
    fn process_test_sequence(#[default("125 17")] input: &str) -> Sequence {
        parse_input(input).unwrap()
    }

    #[rstest]
    #[case("253000 1 7", 1)]
    #[case("253 0 2024 14168", 2)]
    #[case("512072 1 20 24 28676032", 3)]
    #[case("512 72 2024 2 0 2 4 2867 6032", 4)]
    #[case("1036288 7 2 20 24 4048 1 4048 8096 28 67 60 32", 5)]
    #[case(
        "2097446912 14168 4048 2 0 2 4 40 48 2024 40 48 80 96 2 8 6 7 6 0 3 2",
        6
    )]
    fn test_process_sequence(
        #[case] output_str: &str,
        #[case] count: usize,
        #[with(output_str)] process_test_sequence: Sequence,
    ) -> miette::Result<()> {
        let input = parse_input("125 17")?;
        assert_eq!(
            process_test_sequence.elements,
            process_sequence(&input, count)?
        );
        Ok(())
    }

    #[test]
    fn test_parser() -> miette::Result<()> {
        let input = "1 2024 1 0 9 9 2021976";
        let sequence = parse_input(input)?;
        assert_eq!(
            vec![
                Element::new(1),
                Element::new(2024),
                Element::new(1),
                Element::new(0),
                Element::new(9),
                Element::new(9),
                Element::new(2021976)
            ],
            sequence.elements
        );
        Ok(())
    }

    #[test]
    fn test_element_get_digits() -> miette::Result<()> {
        let element = Element::new(12345);
        assert_eq!((vec![1, 2, 3, 4, 5], 5), element.get_digits()?);
        Ok(())
    }

    #[test]
    fn test_element_split_digits() -> miette::Result<()> {
        let element = Element::new(1234);
        assert_eq!(
            vec![Element::new(12), Element::new(34)],
            element.split_digits()?
        );

        let element = Element::new(100000);
        assert_eq!(
            vec![Element::new(100), Element::new(0)],
            element.split_digits()?
        );
        Ok(())
    }
}
