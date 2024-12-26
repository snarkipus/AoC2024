use day_21::part1::process;
use miette::Context;

#[tracing::instrument]
fn main() -> miette::Result<()> {
    tracing_subscriber::fmt::init();
    todo!();
    let file = include_str!("../../input1.txt");
    let result = process(file).context("process part 1")?;
    println!("{:#?}", result);
    Ok(())
}
