use backend::models::api::{ClientMessage, ServerMessage};
use schemars::schema_for;
use std::{fs, path::Path};

fn main() {
    // Get the workspace root (2 levels up from this crate)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).parent().unwrap().parent().unwrap();
    let output_dir = workspace_root.join("packages/shared/schema");

    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    // Generate schemas for WebSocket types
    let schemas = vec![
        ("ClientMessage", schema_for!(ClientMessage)),
        ("ServerMessage", schema_for!(ServerMessage)),
    ];

    // Write schemas to separate files (they'll be combined in TypeScript generation)
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
