use std::io;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let save = serde_json::from_reader(io::stdin())?;
    println!("{}", cookieclicker_save::encode(&save));

    Ok(())
}
