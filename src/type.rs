// #[derive(Debug, Clone)]
// pub enum DataTypeVariant {
//     U32,
//     SLICE(Box<DataTypeVariant>),
// }

use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum DataType {
    U32,
    SLICE(Box<DataType>),
    DYNAMIC_OBJECT(Rc<String>),
}

// impl DataType {
//     pub fn new(variant: DataTypeVariant) -> Self {
//         return DataType { variant };
//     }
// }
