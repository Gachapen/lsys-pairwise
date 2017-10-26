use bson::{from_bson, Bson};
use mongodb::{self, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use na::{DMatrix, DVector};
use serde_enum;
use std::collections::HashMap;
use std::iter::FromIterator;

use db;
use cfg;
use model::{Metric, Weighting};

#[derive(Serialize, Deserialize)]
pub struct SampleWeight {
    name: String,
    weight: f32,
}

struct SampleSet {
    num: usize,
    names: Vec<String>,
    index_map: HashMap<String, usize>,
}

pub fn calculate_sample_weights(
    token: &str,
    metric: &Metric,
    db_client: &mongodb::Client,
) -> Vec<SampleWeight> {
    let db = db_client.db(db::NAME);

    let sample_set = get_sample_set(&db);
    let mut weight_matrix = make_weight_matrix(token, metric, &sample_set, &db);
    normalize_weight_matrix(&mut weight_matrix, sample_set.num);
    let criteria_weights = calculate_criteria_weights(&weight_matrix, sample_set.num);

    make_sample_weights(&criteria_weights, &sample_set.names)
}

pub fn print_stats(token: &str, metric: &Metric, cfg: &cfg::Db) {
    let db_client = db::connect(&cfg);
    let db = db_client.db(db::NAME);

    let sample_set = get_sample_set(&db);
    println!("N: {}", sample_set.num);

    let mut weight_matrix = make_weight_matrix(token, metric, &sample_set, &db);
    println!("Weight matrix: {}", weight_matrix);

    normalize_weight_matrix(&mut weight_matrix, sample_set.num);
    println!("Normalized weight matrix: {}", weight_matrix);

    let criteria_weights = calculate_criteria_weights(&weight_matrix, sample_set.num);
    println!("Criteria weights: {}", criteria_weights);

    let sample_weights = make_sample_weights(&criteria_weights, &sample_set.names);
    for &SampleWeight { ref name, weight } in sample_weights.iter() {
        println!("{} <= {}", name, weight);
    }
}

fn get_sample_set(db: &Database) -> SampleSet {
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
    let num = samples.len();

    let names: Vec<_> = samples.into_iter().map(|sample| sample.name).collect();

    let index_map = HashMap::<String, usize>::from_iter(
        names.iter().enumerate().map(|(i, name)| (name.clone(), i)),
    );

    SampleSet {
        num: num,
        index_map: index_map,
        names: names,
    }
}

fn make_weight_matrix(
    token: &str,
    metric: &Metric,
    sample_set: &SampleSet,
    db: &Database,
) -> DMatrix<f32> {
    let SampleSet {
        num, ref index_map, ..
    } = *sample_set;
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

    let mut weight_matrix = DMatrix::<f32>::identity(num, num);
    for weight in weights {
        let col = index_map[weight.a.as_str()];
        let row = index_map[weight.b.as_str()];
        weight_matrix
            .columns_mut(col, 1)
            .rows_mut(row, 1)
            .fill(weight.weight);
        weight_matrix
            .columns_mut(row, 1)
            .rows_mut(col, 1)
            .fill(1.0 / weight.weight);
    }

    weight_matrix
}

fn normalize_weight_matrix(weight_matrix: &mut DMatrix<f32>, num: usize) {
    // Normalize on columns
    for col in 0..num {
        let sum: f32 = weight_matrix.columns(col, 1).iter().sum();
        let normalization = 1.0 / sum;
        for weight in weight_matrix.columns_mut(col, 1).iter_mut() {
            *weight *= normalization;
        }
    }
}

fn calculate_criteria_weights(weight_matrix: &DMatrix<f32>, num: usize) -> DVector<f32> {
    // Calculate mean of each row
    DVector::<f32>::from_iterator(
        num,
        (0..num).map(|row| {
            weight_matrix.rows(row, 1).iter().sum::<f32>() / num as f32
        }),
    )
}

fn make_sample_weights(criteria_weights: &DVector<f32>, names: &[String]) -> Vec<SampleWeight> {
    let mut sample_weights: Vec<_> = criteria_weights
        .iter()
        .enumerate()
        .map(|(i, w)| {
            SampleWeight {
                name: names[i].clone(),
                weight: *w,
            }
        })
        .collect();
    sample_weights.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
    sample_weights
}
