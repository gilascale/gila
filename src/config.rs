use deepsize::DeepSizeOf;

#[derive(Debug, DeepSizeOf)]
pub struct Config {
    pub max_memory: usize,
}
