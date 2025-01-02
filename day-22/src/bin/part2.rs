use day_22::part2::process;
use miette::Context;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[tracing::instrument]
fn main() -> miette::Result<()> {
    init();

    let file = include_str!("../../input2.txt");
    let result = process(file).context("process part 2")?;
    println!("{}", result);
    Ok(())
}

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("day_22=debug".parse().unwrap()),
        )
        .with_span_events(FmtSpan::NONE)
        .try_init();
}
