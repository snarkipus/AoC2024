use day_11::part1_claude::process;
use miette::Context;

#[tracing::instrument]
fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();

    let file = include_str!("../../input1.txt");
    let result = process(file, 25).context("process part 1")?;
    println!("{}", result);
    Ok(())
}
