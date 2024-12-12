use itertools::Itertools;
use miette::{IntoDiagnostic, Result, miette};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Element {
    value: usize,
}

impl Element {
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
        let (digits, length) = self.get_digits()?;
        let left = digits
            .iter()
            .take(&length / 2)
            .join("")
            .parse::<usize>()
            .into_diagnostic()?;
        let right = digits
            .iter()
            .skip(&length / 2)
            .join("")
            .parse::<usize>()
            .into_diagnostic()?;
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
pub fn process(input: &str, blink_count: usize) -> miette::Result<String> {
    let input_sequence = parse_input(input)?;
    let result = process_sequence(&input_sequence, blink_count)?;

    Ok(result.len().to_string())
}

fn parse_input(input: &str) -> Result<Sequence> {
    let elements: Vec<Element> = input
        .split_whitespace()
        .map(|x| x.parse::<usize>().unwrap())
        .map(Element::new)
        .collect::<Vec<_>>();
    Ok(Sequence { elements })
}

fn process_sequence(input_sequence: &Sequence, count: usize) -> Result<Vec<Element>> {
    if count == 0 {
        return Ok(input_sequence.elements.clone());
    }

    let mut new_elements = Vec::new();
    for element in &input_sequence.elements {
        if element.is_zero()? {
            new_elements.push(Element::new(1));
        } else if element.is_even()? {
            let mut split_elements = element.split_digits()?;
            new_elements.append(&mut split_elements);
        } else {
            new_elements.push(Element::new(element.value * 2024));
        }
    }

    // Create a new sequence from the transformed elements
    let new_sequence = Sequence { elements: new_elements };
    
    // Recursively process the new sequence
    process_sequence(&new_sequence, count - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log;
    use rstest::{fixture, rstest};

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
    #[case("2097446912 14168 4048 2 0 2 4 40 48 2024 40 48 80 96 2 8 6 7 6 0 3 2", 6)]
    fn test_process_sequence(
            #[case] output_str: &str,
            #[case] count: usize,
            #[with(output_str)] process_test_sequence: Sequence,

    ) -> miette::Result<()> {
        let input = parse_input("125 17")?;
        assert_eq!(process_test_sequence.elements, process_sequence(&input, count)?);
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
