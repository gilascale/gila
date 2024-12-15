// #[derive(Debug, Clone)]
// pub enum DataTypeVariant {
//     U32,
//     SLICE(Box<DataTypeVariant>),
// }

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    U32,
    STRING,
    SLICE(Box<DataType>),
    DYNAMIC_OBJECT(Rc<String>),
    GENERIC(Rc<String>),
}

impl DataType {
    pub fn assignable_from(self, other: Self) -> bool {
        return self == other;
    }
}
