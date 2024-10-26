use hora::core::{ann_index::ANNIndex, metrics::Metric};
use hora::index::{hnsw_idx::HNSWIndex, hnsw_params::HNSWParams};

alloy_sol_types::sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        uint32 k;
        uint32[] dest;
    }
}

/// Create a HNSW index from `samples` with `DotProduct` metric, and query it with a given `query` vector.
///
/// HNSW and dot-product is chosen as they turn out to be the least demanding for the zkVM.
///
/// Returns the indices of the top `top_k` samples in the index.
pub fn index_and_query(samples: Vec<Vec<f32>>, query: Vec<f32>, top_k: u32) -> Vec<u32> {
    // let raw_samples = include_bytes!("../../data/foods-smol.json");
    // let x = Vec::<Vec<f32>>::fr(raw_samples.iter());
    // ensure each sample has the same dimension as the query
    let len = query.len();
    for sample in &samples {
        assert_eq!(sample.len(), len);
    }

    // create & add samples to index
    let mut index = HNSWIndex::<f32, u32>::new(len, &HNSWParams::<f32>::default());
    for (i, sample) in samples.iter().enumerate() {
        index.add(sample, i as u32).unwrap();
    }

    // construct HNSW
    index.build(Metric::DotProduct).unwrap();

    // make a query
    index.search(&query, top_k as usize)
}
