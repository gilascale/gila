use deepsize::DeepSizeOf;

#[derive(Clone, Debug, DeepSizeOf)]
pub struct Config {
    pub max_memory: usize,
    pub gc_threshold: f64,
}
