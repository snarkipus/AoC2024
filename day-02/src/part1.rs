use miette::IntoDiagnostic;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Slope {
    Increasing,
    Decreasing,
    Unsafe,
}

pub fn evaluate_slope(start: i32, end: i32) -> Slope {
    match (start, end) {
        (start, end) if start < end && end - start >= 1 && end - start <= 3 => Slope::Increasing,
        (start, end) if start > end && start - end >= 1 && start - end <= 3 => Slope::Decreasing,
        _ => Slope::Unsafe,
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let data: Vec<Vec<i32>> = input
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|n| n.parse::<i32>().into_diagnostic())
                .collect::<Result<Vec<i32>, _>>()
        })
        .collect::<Result<Vec<Vec<i32>>, _>>()?;

    let safe_count = data
        .iter()
        .filter(|report| {
            let initial_slope = evaluate_slope(report[0], report[1]);
            if initial_slope == Slope::Unsafe {
                return false;
            }

            let mut prev_slope = initial_slope;
            report.windows(2).all(|window| {
                let current_slope = evaluate_slope(window[0], window[1]);
                let is_valid = current_slope != Slope::Unsafe && current_slope == prev_slope;
                prev_slope = current_slope;
                is_valid
            })
        })
        .count();

    Ok(safe_count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";
        assert_eq!("2", process(input)?);
        Ok(())
    }
}
