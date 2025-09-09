use thiserror::Error;

/// Parse error type
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse: {0}")]
    ParseFailed(String),
    
    #[error("UTF-8 conversion error")]
    Utf8Error(#[from] std::str::Utf8Error),
}

/// Main error type for the engine
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    
    #[error("Parse error in {file}: {message}")]
    ParseError { file: String, message: String },

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Graph processing error: {0}")]
    GraphError(String),

    #[error("Analytics error: {0}")]
    AnalyticsError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Service not initialized")]
    ServiceNotInitialized,
}

/// Result type alias for engine operations
pub type EngineResult<T> = Result<T, EngineError>;