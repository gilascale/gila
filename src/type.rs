// #[derive(Debug, Clone)]
// pub enum DataTypeVariant {
//     U32,
//     SLICE(Box<DataTypeVariant>),
// }

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    VOID,
    ANY,
    U32,
    I32,
    I64,
    F32,
    F64,
    BOOL,
    STRING,
    FN(Vec<DataType>, Box<DataType>),
    SLICE(Box<DataType>),
    NAMED_REFERENCE(Rc<String>),
    DYNAMIC_OBJECT(Vec<DataType>),
    GENERIC(Rc<String>),
}

impl DataType {
    pub fn assignable_from(self, other: Self) -> bool {
        return self == other;
    }
}
