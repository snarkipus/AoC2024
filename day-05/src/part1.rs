use miette::*; 
struct Rule {
    page: usize,
    constraint: Option<Vec<usize>>,
}

impl Rule {
    fn new(page: usize) -> Self {
        Self {
            page,
            constraint: None,
        }
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let data = input
        .lines()
        .collect::<Vec<&str>>();

    let [rules, updates] = data
        .split(|line| line.is_empty())
        .collect::<Vec<_>>()[..]
        else { return Err(miette!("Invalid input")) };

    dbg!(rules, updates);

    Ok("142".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47";
        assert_eq!("143", process(input)?);
        Ok(())
    }
}
