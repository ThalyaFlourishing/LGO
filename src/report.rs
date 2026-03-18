use std::fs;

pub fn run(input_file: &str) -> Result<(), String> {
    let input = fs::read_to_string(input_file)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let report = generate_report(&input);

    println!("Report for '{}':\n{}", input_file, report);

    Ok(())
}

fn generate_report(input: &str) -> String {
    let line_count = input.lines().count();
    let word_count = input.split_whitespace().count();
    let char_count = input.chars().count();

    format!(
        "Lines: {}\nWords: {}\nCharacters: {}",
        line_count, word_count, char_count
    )
}