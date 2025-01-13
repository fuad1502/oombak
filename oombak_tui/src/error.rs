pub type OombakResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
