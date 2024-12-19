pub type OmbakResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
