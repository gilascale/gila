// #[derive(Debug, Clone)]
// pub enum DataTypeVariant {
//     U32,
//     SLICE(Box<DataTypeVariant>),
// }

#[derive(Debug, Clone)]
pub enum DataType {
    U32,
    SLICE(Box<DataType>),
}

// impl DataType {
//     pub fn new(variant: DataTypeVariant) -> Self {
//         return DataType { variant };
//     }
// }
