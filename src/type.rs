#[derive(Debug)]
pub enum DataTypeVariant {
    U32,
}

#[derive(Debug)]
pub struct DataType {
    pub variant: DataTypeVariant,
}

impl DataType {
    pub fn new(variant: DataTypeVariant) -> Self {
        return DataType { variant };
    }
}
