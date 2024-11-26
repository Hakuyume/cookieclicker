use std::io;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let value = cookieclicker_save::decode(line.trim())?;
    println!("{}", serde_json::to_string(&value)?);

    Ok(())
}
