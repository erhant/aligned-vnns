use ollama_rs::{generation::embeddings::request::GenerateEmbeddingsRequest, Ollama};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    name: String,
    description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddedData<T> {
    pub data: T,
    pub embeddings: Vec<f32>,
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

pub async fn index(path: &str, model: &str) {
    let ollama = Ollama::default();

    // read data
    println!("Reading data from: {}", path);
    let path = Path::new(path);
    let data_bytes = fs::read(path).await.expect("Unable to read file");
    let data = serde_json::from_slice::<Vec<Data>>(&data_bytes).unwrap();
    assert!(!data.is_empty(), "no data found");

    // convert to texts
    let texts = data
        .iter()
        .map(|food| food.to_string())
        .collect::<Vec<String>>();

    // generate embeddings
    println!("Generating embeddings with: {}", model);
    let request = GenerateEmbeddingsRequest::new(model.to_string(), texts.into());
    let res = ollama.generate_embeddings(request).await.unwrap();

    // convert to embedded data
    println!(
        "Exporting embedding data (dim: {})",
        res.embeddings[0].len()
    );
    let embedded_data = data
        .into_iter()
        .zip(res.embeddings)
        .map(|(data, embeddings)| EmbeddedData { data, embeddings })
        .collect::<Vec<EmbeddedData<Data>>>();

    // write embedded data to file
    let output_path = path.with_extension("index.json");
    println!("Writing data to: {:?}", output_path);
    let embedded_data_bytes = serde_json::to_vec(&embedded_data).unwrap();
    fs::write(output_path, embedded_data_bytes)
        .await
        .expect("Unable to write file");
}

pub async fn query(path: &str, text: &str, model: &str) {
    let ollama = Ollama::default();

    // generate embeddings
    let request = GenerateEmbeddingsRequest::new(model.to_string(), vec![text.to_string()].into());
    let res = ollama.generate_embeddings(request).await.unwrap();

    // write embedding data to file
    let output_path = Path::new(path).with_extension("query.json");
    println!("Writing data to: {:?}", output_path);
    let embedded_data_bytes = serde_json::to_vec(&res.embeddings[0]).unwrap();
    fs::write(output_path, embedded_data_bytes)
        .await
        .expect("Unable to write file");
}

pub async fn export(path: &str) {
    // read embedded data
    let path = Path::new(path).with_extension("index.json");
    let data_bytes = fs::read(path).await.expect("Unable to read file");
    let embedded_data = serde_json::from_slice::<Vec<EmbeddedData<Data>>>(&data_bytes).unwrap();
    assert!(!embedded_data.is_empty(), "no embedded data found");

    // extract embeddings
    let embeddings: Vec<Vec<f32>> = embedded_data
        .into_iter()
        .map(|embedded| embedded.embeddings)
        .collect();

    // generate Rust code string
    let mut rust_code = String::from("const SAMPLES: [");

    let embedding_len = embeddings[0].len();
    rust_code.push_str(&format!(
        "[f32; {}]; {}] = [\n",
        embedding_len,
        embeddings.len()
    ));

    for embedding in embeddings {
        rust_code.push_str("    [");
        for (i, value) in embedding.iter().enumerate() {
            rust_code.push_str(&format!("{}", value));
            if i < embedding_len - 1 {
                rust_code.push_str(", ");
            }
        }
        rust_code.push_str("],\n");
    }
    rust_code.push_str("];\n");

    // print Rust code string
    println!("{}", rust_code);
}
