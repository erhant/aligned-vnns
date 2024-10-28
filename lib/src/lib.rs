/// Compute the best sample from a list of samples given a query.
///
/// Uses the dot product to compute the similarity between the samples and the query.
/// Assumes that the samples and the query have the same length, and the input values are
/// scale-invariant and within the range [-1, 1].
pub fn compute_best_sample(samples: &[Vec<f32>], query: &[f32]) -> usize {
    samples
        .iter()
        .map(|sample| {
            sample
                .iter()
                .zip(query)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                .sqrt()
        })
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap()
        .0
}

pub fn iterative_similarity_search(
    samples: Vec<Vec<f32>>,
    query: Vec<f32>,
    batch_size: usize,
) -> (usize, Vec<f32>) {
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

    let idx = compute_best_sample(&current_samples, &query);
    let result = current_samples[idx].clone();

    (idx, result)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_compute_best_sample() {
        let samples = vec![
            vec![0.101, 0.201, 0.301],
            vec![0.401, 0.501, 0.601],
            vec![0.700, 0.800, 0.900],
        ];
        let query = vec![0.1, 0.2, 0.3];
        assert_eq!(compute_best_sample(&samples, &query), 0);
    }

    #[test]
    fn test_iterative_similarity_search() {
        let samples = vec![
            vec![0.1, 0.2, 0.3],
            vec![0.4, 0.5, 0.6],
            vec![0.7, 0.8, 0.9],
            vec![0.10, 0.11, 0.12],
            vec![0.13, 0.14, 0.15],
            vec![0.16, 0.17, 0.18],
        ];
        let query = vec![0.99, 0.99, 0.99];
        assert_eq!(
            iterative_similarity_search(samples, query, 2).1,
            vec![0.4, 0.5, 0.6]
        );
    }
}
