alloy_sol_types::sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        uint32 idx; // index of the most similar sample
    }
}

pub fn compute_best_sample(samples: &[Vec<f32>], query: &[f32]) -> usize {
    samples
        .iter()
        .map(|sample| sample.iter().zip(query).map(|(a, b)| a * b).sum::<f32>())
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap()
        .0
}

pub fn iterative_similarity_search(
    samples: Vec<Vec<f32>>,
    query: Vec<f32>,
    batch_size: usize,
) -> usize {
    let mut current_samples = samples;
    while current_samples.len() > batch_size {
        let mut best_samples = Vec::new();
        for chunk in current_samples.chunks(batch_size) {
            best_samples.push(compute_best_sample(chunk, &query));
        }
        current_samples = best_samples
            .iter()
            .map(|&idx| current_samples[idx].clone())
            .collect::<Vec<_>>();
    }

    compute_best_sample(&current_samples, &query)
}
