//! Standalone wiki lookup test.
//! Run with: cargo run --bin wikitest

fn main() {
    let name = "Umbari Robe of Beasts";
    let page_title = format!("Item:{}", name.replace(' ', "_"));
    let url = format!(
        "https://lotro-wiki.com/api.php?action=parse&page={}&prop=wikitext&format=json",
        page_title
    );

    println!("Requesting URL:\n  {}\n", url);

    let response = ureq::get(&url)
        .set("User-Agent", "lgo-wikitest/1.0")
        .call();

    match response {
        Err(e) => {
            eprintln!("HTTP error: {}", e);
        }
        Ok(resp) => {
            println!("HTTP status: {}", resp.status());
            match resp.into_string() {
                Err(e) => eprintln!("Failed to read body: {}", e),
                Ok(body) => {
                    println!("Response body:\n{}", body);
                }
            }
        }
    }
}