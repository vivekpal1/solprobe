use std::io::{self, Write};

pub fn get_url_input() -> Result<String, io::Error> {
    print!("Enter Solana RPC URL (or press Enter for default): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let url = input.trim();
    if url.is_empty() {
        Ok("https://api.mainnet-beta.solana.com".to_string())
    } else {
        Ok(url.to_string())
    }
}

pub fn get_interval_input() -> Result<u64, io::Error> {
    print!("Enter update interval in seconds (default is 5): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let interval = input.trim();
    if interval.is_empty() {
        Ok(5)
    } else {
        match interval.parse() {
            Ok(val) => Ok(val),
            Err(_) => {
                println!("Invalid input. Using default interval of 5 seconds.");
                Ok(5)
            }
        }
    }
}