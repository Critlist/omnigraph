pub mod centrality;
pub mod community;
pub mod quality;
pub mod risk;

use anyhow::Result;
use og_graph::graph::CodeGraph;
use std::collections::HashMap;

/// Value types for metrics
#[derive(Debug, Clone)]
pub enum MetricValue {
    Float(f64),
    Integer(i64),
    Vector(Vec<f64>),
    Map(HashMap<String, f64>),
}

impl MetricValue {
    pub fn as_float(&self) -> Option<f64> {
        match self {
            MetricValue::Float(v) => Some(*v),
            MetricValue::Integer(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, f64>> {
        match self {
            MetricValue::Map(m) => Some(m),
            _ => None,
        }
    }
}

/// Results from a metric calculation
#[derive(Debug, Clone)]
pub struct MetricResults {
    pub name: String,
    pub values: HashMap<String, MetricValue>,
}

impl MetricResults {
    pub fn new(name: String) -> Self {
        Self {
            name,
            values: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, key: String, value: MetricValue) {
        self.values.insert(key, value);
    }

    pub fn get_node_value(&self, node_id: &str, metric: &str) -> Option<f64> {
        let key = format!("{}_{}", node_id, metric);
        self.values.get(&key)?.as_float()
    }
}

/// Trait for all metrics
pub trait Metric: Send + Sync {
    /// Calculate the metric for a graph
    fn calculate(&self, graph: &CodeGraph) -> Result<MetricResults>;
    
    /// Name of the metric
    fn name(&self) -> &str;
}

/// Normalize a value to 0-1 range
pub fn normalize(value: f64, min: f64, max: f64) -> f64 {
    if max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

/// Calculate percentile rank of a value
pub fn percentile_rank(value: f64, values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    
    // Filter out NaN and infinite values
    let mut sorted: Vec<f64> = values
        .iter()
        .filter(|v| v.is_finite())
        .copied()
        .collect();
    
    if sorted.is_empty() {
        return 0.0;
    }
    
    // Safe sorting with proper NaN handling
    sorted.sort_by(|a, b| {
        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Handle NaN or infinite input value
    if !value.is_finite() {
        return 0.0;
    }
    
    let position = sorted.iter().position(|&v| v >= value).unwrap_or(sorted.len());
    position as f64 / sorted.len() as f64
}