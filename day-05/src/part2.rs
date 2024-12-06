use miette::*;
use std::collections::HashMap;

type PageNumber = usize;
type Rules = HashMap<PageNumber, Vec<PageNumber>>;

const RULE_SEPARATOR: char = '|';

/// Process a string input containing page transition rules and updates.
/// Returns the sum of middle elements from valid/fixed page sequences.
#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let data = input.lines().collect::<Vec<&str>>();

    let [rules, updates] = data.split(|line| line.is_empty()).collect::<Vec<_>>()[..] else {
        return Err(miette!("Invalid input format - expected rules and updates separated by empty line"));
    };

    let pre_rules = create_rules(rules, false)?;
    let post_rules = create_rules(rules, true)?;
    let invalid_updates = check_updates(updates, &pre_rules, &post_rules)?;
    let fixed_updates = fix_updates(&invalid_updates, &pre_rules, &post_rules)?;

    let total = fixed_updates
        .iter()
        .map(|update| update[update.len() / 2])
        .sum::<PageNumber>();

    Ok(total.to_string())
}

/// Creates a HashMap of page transition rules from string input.
/// If reverse is true, swaps the key/value relationship.
fn create_rules(data: &[&str], reverse: bool) -> Result<Rules, Report> {
    let mut rules: Rules = HashMap::new();

    for rule in data {
        let parts: Vec<_> = rule.split(RULE_SEPARATOR).collect();
        if parts.len() != 2 {
            return Err(miette!("Invalid rule format - expected 'number|number'"));
        }

        let (key, value) = if reverse {
            (parts[1], parts[0])
        } else {
            (parts[0], parts[1])
        };

        let key = key.parse::<PageNumber>().into_diagnostic()?;
        let value = value.parse::<PageNumber>().into_diagnostic()?;
        rules.entry(key).or_default().push(value);
    }

    Ok(rules)
}

fn check_updates(
    data: &[&str],
    pre_rules: &Rules,
    post_rules: &Rules,
) -> Result<Vec<Vec<usize>>, Report> {
    let invalid_updates = data
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
                None
            } else {
                Some(update)
            }
        })
        .collect::<Vec<Vec<usize>>>();

    Ok(invalid_updates)
}

#[tracing::instrument]
fn fix_updates(
    invalid_updates: &[Vec<usize>],
    pre_rules: &Rules,
    post_rules: &Rules,
) -> Result<Vec<Vec<usize>>, Report> {
    let fixed_updates = invalid_updates
        .iter()
        .map(|update| {
            let mut invalid = update.to_vec();
            let mut was_fixed = true;

            while was_fixed {
                was_fixed = false;
                for i in 0..invalid.len() - 1 {
                    let valid = pre_rules
                        .get(&invalid[i])
                        .map_or(true, |constraints| constraints.contains(&invalid[i + 1]))
                        && post_rules
                            .get(&invalid[i + 1])
                            .map_or(true, |constraints| constraints.contains(&invalid[i]));

                    if !valid {
                        invalid.swap(i, i + 1);
                        was_fixed = true;
                    }
                }
            }
            
            invalid // Return sequence whether fixed or not
        })
        .collect();

    Ok(fixed_updates)
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
        assert_eq!("123", process(input)?);
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

    #[test]
    fn test_process2() -> miette::Result<()> {
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

97,13,75,29,47";
        assert_eq!("47", process(input)?);
        Ok(())
    }
}
