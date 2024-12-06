use miette::*;
use std::collections::HashMap;

type Rules = HashMap<usize, Vec<usize>>;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let data = input.lines().collect::<Vec<&str>>();

    let [rules, updates] = data.split(|line| line.is_empty()).collect::<Vec<_>>()[..] else {
        return Err(miette!("Invalid input"));
    };

    let pre_rules = create_rules(rules, false)?;
    let post_rules = create_rules(rules, true)?;
    let valid_updates = check_updates(updates, &pre_rules, &post_rules)?;

    let total = valid_updates
        .iter()
        .map(|update| update[update.len() / 2])
        .sum::<usize>();

    Ok(total.to_string())
}

fn create_rules(data: &[&str], reverse: bool) -> Result<Rules, Report> {
    let mut rules: Rules = HashMap::new();

    for rule in data {
        let parts: Vec<_> = rule.split('|').collect();
        if parts.len() != 2 {
            return Err(miette!("Invalid rule"));
        }

        let (key, value) = if reverse {
            (parts[1], parts[0])
        } else {
            (parts[0], parts[1])
        };

        let key = key.parse::<usize>().into_diagnostic()?;
        let value = value.parse::<usize>().into_diagnostic()?;
        rules.entry(key).or_default().push(value);
    }

    Ok(rules)
}

#[tracing::instrument(skip(data, pre_rules, post_rules))]
fn check_updates(
    data: &[&str],
    pre_rules: &Rules,
    post_rules: &Rules,
) -> Result<Vec<Vec<usize>>, Report> {
    let valid_updates = data
        .iter()
        .filter_map(|update_str| {
            let update = update_str
                .split(',')
                .map(|n| n.parse::<usize>().into_diagnostic())
                .collect::<Result<Vec<usize>, _>>()
                .ok()?;

            let is_valid = update.windows(2).all(|window| {
                if let [page_1, page_2] = window {
                    pre_rules
                        .get(page_1)
                        .map_or(true, |constraints| constraints.contains(page_2))
                        && post_rules
                            .get(page_2)
                            .map_or(true, |constraints| constraints.contains(page_1))
                } else {
                    true
                }
            });

            if is_valid {
                Some(update)
            } else {
                None
            }
        })
        .collect::<Vec<Vec<usize>>>();

    Ok(valid_updates)
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

    #[test]
    fn test_create_rules() -> miette::Result<()> {
        let input = ["1|2", "1|3", "2|3", "2|4", "3|4", "4|5"];
        let expected = {
            let mut rules = HashMap::new();
            rules.insert(1, vec![2, 3]);
            rules.insert(2, vec![3, 4]);
            rules.insert(3, vec![4]);
            rules.insert(4, vec![5]);
            rules
        };

        assert_eq!(expected, create_rules(&input, false)?);
        Ok(())
    }
}
