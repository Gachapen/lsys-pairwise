use bson::{from_bson, Bson};
use bson::oid::ObjectId;
use mongodb::{self, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use na::{DMatrix, DVector};
use serde_enum;

use db::{self, Weighting};
use cfg;
use model::Metric;

#[derive(Serialize, Deserialize)]
pub struct SampleWeight {
    name: String,
    weight: f32,
}

#[derive(Debug)]
struct SampleSet {
    num: usize,
    ids: Vec<ObjectId>,
}

pub fn calculate_sample_weights(
    task: &str,
    token: &str,
    metric: &Metric,
    db_client: &mongodb::Client,
) -> Vec<SampleWeight> {
    let db = db_client.db(db::NAME);

    let sample_set = get_sample_set(task, &db);
    let mut weight_matrix = make_weight_matrix(token, metric, &sample_set, &db);
    normalize_weight_matrix(&mut weight_matrix, sample_set.num);
    let criteria_weights = calculate_criteria_weights(&weight_matrix, sample_set.num);

    make_sample_weights(&criteria_weights, &sample_set.ids)
}

pub fn print_stats(task: &str, token: &str, metric: &Metric, cfg: &cfg::Db) {
    let db_client = db::connect(&cfg);
    let db = db_client.db(db::NAME);

    let sample_set = get_sample_set(task, &db);
    println!("N: {}", sample_set.num);

    let mut weight_matrix = make_weight_matrix(token, metric, &sample_set, &db);
    println!("Weight matrix: {}", weight_matrix);

    normalize_weight_matrix(&mut weight_matrix, sample_set.num);
    println!("Normalized weight matrix: {}", weight_matrix);

    let criteria_weights = calculate_criteria_weights(&weight_matrix, sample_set.num);
    println!("Criteria weights: {}", criteria_weights);

    let sample_weights = make_sample_weights(&criteria_weights, &sample_set.ids);
    for &SampleWeight { ref name, weight } in sample_weights.iter() {
        println!("{} <= {}", name, weight);
    }
}

fn get_sample_set(task: &str, db: &Database) -> SampleSet {
    let sample_docs: Vec<_> = db.collection(db::COLLECTION_SAMPLE)
        .find(Some(doc!{ "task": task }), None)
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let ids: Vec<ObjectId> = sample_docs
        .into_iter()
        .map(|doc| {
            doc.get_object_id("_id")
                .expect("Failed deserializing documents")
                .clone()
        })
        .collect();
    let num = ids.len();

    SampleSet { num: num, ids: ids }
}

fn make_weight_matrix(
    token: &str,
    metric: &Metric,
    sample_set: &SampleSet,
    db: &Database,
) -> DMatrix<f32> {
    let SampleSet { num, ref ids, .. } = *sample_set;

    let metric_str = serde_enum::to_string(&metric).unwrap();

    let mut weight_matrix = DMatrix::<f32>::identity(num, num);
    for col in 0..num {
        for row in (col + 1)..num {
            let a = &ids[col];
            let b = &ids[row];

            let doc = db.collection(db::COLLECTION_WEIGHT)
                .find_one(
                    Some(doc! {
                        "token": token,
                        "metric": &metric_str,
                        "a": a.clone(),
                        "b": b.clone(),
                    }),
                    None,
                )
                .expect("Failed quering DB");

            if let Some(doc) = doc {
                // Weight was found in column major order.
                let weight: Weighting =
                    from_bson(Bson::from(doc)).expect("Failed deserializing weight");
                weight_matrix
                    .columns_mut(col, 1)
                    .rows_mut(row, 1)
                    .fill(weight.weight);
                weight_matrix
                    .columns_mut(row, 1)
                    .rows_mut(col, 1)
                    .fill(1.0 / weight.weight);
            } else {
                // Alternatively weight is in row major order.
                let doc = db.collection(db::COLLECTION_WEIGHT)
                    .find_one(
                        Some(doc! {
                            "token": token,
                            "metric": &metric_str,
                            "a": b.clone(),
                            "b": a.clone(),
                        }),
                        None,
                    )
                    .expect("Failed quering DB")
                    .expect("Missing weight");

                let weight: Weighting =
                    from_bson(Bson::from(doc)).expect("Failed deserializing weight");
                weight_matrix
                    .columns_mut(col, 1)
                    .rows_mut(row, 1)
                    .fill(1.0 / weight.weight);
                weight_matrix
                    .columns_mut(row, 1)
                    .rows_mut(col, 1)
                    .fill(weight.weight);
            }
        }
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

fn make_sample_weights(criteria_weights: &DVector<f32>, ids: &[ObjectId]) -> Vec<SampleWeight> {
    let mut sample_weights: Vec<_> = criteria_weights
        .iter()
        .enumerate()
        .map(|(i, w)| {
            SampleWeight {
                name: ids[i].to_hex(),
                weight: *w,
            }
        })
        .collect();
    sample_weights.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
    sample_weights
}
