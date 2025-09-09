use std::sync::Arc;

/// Trait for reporting progress of long-running operations
pub trait ProgressReporter: Send + Sync {
    /// Report progress with a message and percentage
    fn report(&self, message: &str, percentage: f32);
    
    /// Mark operation as complete
    fn complete(&self, message: Option<&str>);
    
    /// Report an error
    fn error(&self, message: &str, error: Option<&dyn std::error::Error>);
}

/// No-op progress reporter for when progress reporting is not needed
pub struct NoOpProgressReporter;

impl ProgressReporter for NoOpProgressReporter {
    fn report(&self, _message: &str, _percentage: f32) {}
    fn complete(&self, _message: Option<&str>) {}
    fn error(&self, _message: &str, _error: Option<&dyn std::error::Error>) {}
}

/// Console progress reporter for CLI usage
pub struct ConsoleProgressReporter;

impl ProgressReporter for ConsoleProgressReporter {
    fn report(&self, message: &str, percentage: f32) {
        println!("[{:3.0}%] {}", percentage, message);
    }
    
    fn complete(&self, message: Option<&str>) {
        println!("✅ {}", message.unwrap_or("Complete"));
    }
    
    fn error(&self, message: &str, error: Option<&dyn std::error::Error>) {
        if let Some(err) = error {
            eprintln!("❌ {}: {}", message, err);
        } else {
            eprintln!("❌ {}", message);
        }
    }
}

/// Create an Arc-wrapped progress reporter
pub fn create_progress_reporter(console: bool) -> Arc<dyn ProgressReporter> {
    if console {
        Arc::new(ConsoleProgressReporter)
    } else {
        Arc::new(NoOpProgressReporter)
    }
}