use tanoshi_vm::bus::ExtensionBus;

use crate::data::Index;

pub async fn generate_json(bus: ExtensionBus) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("repo");
    let _ = std::fs::create_dir_all(path.join("library"));

    let sources = bus
        .list()
        .await?
        .iter()
        .map(|source| Index {
            path: format!("library/{}.wasm", source.name),
            id: source.id,
            name: source.name.clone(),
            version: source.version.to_string(),
            lib_version: source.lib_version.to_string(),
            icon: source.icon.clone(),
        })
        .collect::<Vec<Index>>();

    let file = std::fs::File::create(path.join("index.json"))?;
    serde_json::to_writer(&file, &sources)?;

    Ok(())
}
