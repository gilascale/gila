#[derive(Debug, Clone)]
pub enum DataTypeVariant {
    U32,
}

#[derive(Debug, Clone)]
pub struct DataType {
    pub variant: DataTypeVariant,
}

impl DataType {
    pub fn new(variant: DataTypeVariant) -> Self {
        return DataType { variant };
    }
}
