use std::io;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let value = serde_json::from_reader(io::stdin())?;
    println!("{}", cookieclicker_save::encode(&value));

    Ok(())
}
