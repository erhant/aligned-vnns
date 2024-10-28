use ollama_rs::{generation::embeddings::request::GenerateEmbeddingsRequest, Ollama};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    name: String,
    description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddedData<T> {
    /// Raw data object.
    pub data: T,
    /// Embedding vector.
    pub embeddings: Vec<f32>,
    /// Hex encoded SHA256 digest.
    pub hash: String,
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
    let texts = data.iter().map(|d| d.to_string()).collect::<Vec<String>>();

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
        .map(|(data, embeddings)| {
            let embeddings_bytes = embeddings
                .iter()
                .flat_map(|f| f.to_ne_bytes())
                .collect::<Vec<_>>();

            EmbeddedData {
                data,
                embeddings,
                hash: hex::encode(Sha256::digest(&embeddings_bytes)),
            }
        })
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
    let embedding = res.embeddings[0].clone();
    println!("Embedding dim: {}", embedding.len());

    // write embedding data to file
    let output_path = Path::new(path).with_extension("query.json");
    println!("Writing data to: {:?}", output_path);
    let embedded_data_bytes = serde_json::to_vec(&embedding).unwrap();
    fs::write(output_path, embedded_data_bytes)
        .await
        .expect("Unable to write file");
}
