use miette::IntoDiagnostic;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let mut a = vec![];
    let mut b = vec![];

    for line in input.lines() {
        let mut cols = line.split_whitespace();
        a.push(cols.next().unwrap().parse::<i32>().into_diagnostic()?);
        b.push(cols.next().unwrap().parse::<i32>().into_diagnostic()?);
    }

    // sort a in ascending order
    a.sort_unstable();

    // sort b in ascending order
    b.sort_unstable();

    // zip a and b together and give the absolute value of the difference
    let result = a
        .iter()
        .zip(b.iter())
        .map(|(a, b)| (a - b).abs())
        .sum::<i32>();

    // return the sum of the absolute differences
    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "3   4
4   3
2   5
1   3
3   9
3   3";
        assert_eq!("11", process(input)?);
        Ok(())
    }
}
