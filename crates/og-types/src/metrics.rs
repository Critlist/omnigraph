use serde::{Deserialize, Serialize};

/// File-level metrics
#[derive(Debug, Clone, Default)]
pub struct FileMetrics {
    pub lines_of_code: usize,
    pub complexity: usize,
    pub functions: usize,
    pub classes: usize,
    pub imports: usize,
    pub exports: usize,
}

/// UI-facing stable contract for node metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UINodeMetricsV1 {
    pub path: String,
    pub name: String,
    pub node_type: String,
    pub community: i64,
    pub importance: f32,
    pub risk: f32,
    pub chokepoint: f32,
    pub payoff: f32,
    pub raw: RawMetrics,
    pub normalized: NormalizedMetrics,
    #[serde(default = "default_version")]
    pub version: u8,
}

fn default_version() -> u8 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawMetrics {
    pub pagerank_imports: f64,
    pub pagerank_calls: Option<f64>,
    pub indegree: i64,
    pub outdegree: i64,
    pub k_core: i64,
    pub clustering: f64,
    pub betweenness: f64,
    pub churn: i64,
    pub complexity: i64,
    pub owners: i64,
    pub coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedMetrics {
    pub pagerank_imports: f64,
    pub pagerank_calls: Option<f64>,
    pub indegree: f64,
    pub k_core: f64,
    pub clustering: f64,
    pub betweenness: f64,
    pub churn: f64,
    pub complexity: f64,
    pub owners: f64,
    pub coverage: f64,
}

/// Composite metric outputs
#[derive(Debug, Clone)]
pub struct CompositeOutputs {
    pub importance: f64,
    pub chokepoint: f64,
    pub risk: f64,
    pub payoff: f64,
}

/// Normalization ranges for metrics
#[derive(Debug, Clone)]
pub struct NormalizationRanges {
    pub pagerank_imports: (f64, f64),
    pub pagerank_calls: (f64, f64),
    pub k_core: (f64, f64),
    pub indegree: (f64, f64),
    pub clustering: (f64, f64),
    pub betweenness: (f64, f64),
    pub churn: (f64, f64),
    pub complexity: (f64, f64),
    pub owners: (f64, f64),
    pub coverage: (f64, f64),
}