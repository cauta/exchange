use backend::models::api::{ClientMessage, ServerMessage};
use schemars::schema_for;
use std::{fs, path::Path};

fn main() {
    // Create the output directory if it doesn't exist
    let output_dir = Path::new("packages/shared/schema");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    // Generate schemas for WebSocket types
    let schemas = vec![
        ("ClientMessage", schema_for!(ClientMessage)),
        ("ServerMessage", schema_for!(ServerMessage)),
    ];

    // Write schemas to files
    for (name, schema) in &schemas {
        let file_path = output_dir.join(format!("{}.json", name));
        fs::write(
            &file_path,
            serde_json::to_string_pretty(&schema).expect("Failed to serialize schema"),
        )
        .expect("Failed to write schema file");
        println!("Generated schema: {}", file_path.display());
    }

    println!("\nSuccessfully generated WebSocket schemas!");
}
