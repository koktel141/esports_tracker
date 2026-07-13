use reqwest::Error;
#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Connecting to the server...");
    let url = "https://api.opendota.com/api/proMatches";
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let text = response.text().await?;
        
        let preview_length = if text.len() > 500 { 500 } else { text.len() };
        println!("Data received successfully!\n\nRaw Data Preview:\n{}", &text[..preview_length]);
    } else {
        println!("Failed to fetch data. Status code: {}", response.status());
    }
    Ok(())
}
