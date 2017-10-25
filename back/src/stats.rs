use bson::{from_bson, Bson};
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;
use na::{DMatrix, DVector};
use serde_enum;
use std::collections::HashMap;
use std::iter::FromIterator;

use db;
use cfg::Config;
use model::{Metric, Weighting};

pub fn print_stats(token: &str, metric: &Metric) {
    let config = Config::from_env();
    let db_client = db::connect(&config.db);
    let db = db_client.db(db::NAME);

    let sample_docs: Vec<_> = db.collection(db::COLLECTION_SAMPLE)
        .find(None, None)
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let samples: Vec<db::Sample> = sample_docs
        .into_iter()
        .map(|doc| from_bson(Bson::from(doc)))
        .collect::<Result<_, _>>()
        .unwrap();
    let num_samples = samples.len();

    println!("N: {}", num_samples);

    let sample_index_map = HashMap::<&str, usize>::from_iter(
        samples
            .iter()
            .enumerate()
            .map(|(i, sample)| (sample.name.as_str(), i)),
    );

    let sample_names: Vec<_> = samples.iter().map(|sample| sample.name.as_str()).collect();

    let weights_cursor = db.collection(db::COLLECTION_WEIGHT)
        .find(
            Some(doc! {
                "token": token,
                "metric": serde_enum::to_string(&metric).unwrap(),
            }),
            None,
        )
        .unwrap();
    let weight_docs: Vec<_> = weights_cursor.collect::<Result<_, _>>().unwrap();
    let weights: Vec<Weighting> = weight_docs
        .into_iter()
        .map(|doc| from_bson(Bson::from(doc)))
        .collect::<Result<_, _>>()
        .unwrap();

    let mut weight_matrix = DMatrix::<f32>::identity(num_samples, num_samples);
    for weight in weights {
        let col = sample_index_map[weight.a.as_str()];
        let row = sample_index_map[weight.b.as_str()];
        weight_matrix
            .columns_mut(col, 1)
            .rows_mut(row, 1)
            .fill(weight.weight);
        weight_matrix
            .columns_mut(row, 1)
            .rows_mut(col, 1)
            .fill(1.0 / weight.weight);
    }
    println!("Weight matrix: {}", weight_matrix);

    // Normalize on columns
    for col in 0..num_samples {
        let sum: f32 = weight_matrix.columns(col, 1).iter().sum();
        let normalization = 1.0 / sum;
        for weight in weight_matrix.columns_mut(col, 1).iter_mut() {
            *weight *= normalization;
        }
    }
    println!("Normalized weight matrix: {}", weight_matrix);

    // Calculate mean of each row
    let criteria_weights = DVector::<f32>::from_iterator(
        num_samples,
        (0..num_samples).map(|row| {
            weight_matrix.rows(row, 1).iter().sum::<f32>() / num_samples as f32
        }),
    );
    println!("Criteria weights: {}", criteria_weights);

    let mut weighted_names: Vec<_> = criteria_weights
        .iter()
        .enumerate()
        .map(|(i, w)| (sample_names[i], w))
        .collect();
    weighted_names.sort_by(|&(_, w1), &(_, w2)| w1.partial_cmp(w2).unwrap());

    for &(name, weight) in weighted_names.iter().rev() {
        println!("{} <= {}", name, weight);
    }
}
