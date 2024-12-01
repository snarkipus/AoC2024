use std::collections::{HashSet, HashMap};

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

    // create a hashset for unique values of column a
    let a_set: HashSet<i32> = a.iter().cloned().collect();

    // create a HashMap with the following:
    //  keys: elements of `a_set`
    //  values: the count of the number of times the key appears in `b` multiplied by the key
    let b_map: HashMap<i32, i32> = a_set.iter()
        .map(|&key| {
            let count = b.iter().filter(|&&value| value == key).count() as i32;
            (key, count * key)
        })
        .collect();

    // using the elements of `a` as the keys, lookup the values from `b_map` and sum them
    let result = a.iter().map(|&key| b_map[&key]).sum::<i32>();

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
        assert_eq!("31", process(input)?);
        Ok(())
    }
}
