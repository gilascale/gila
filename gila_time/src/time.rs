use gila::execution::Object;
use gila::execution::ProcessContext;
use gila::execution::SharedExecutionContext;
use std::{thread, time::Duration};

#[no_mangle]
pub extern "C" fn sleep(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &ProcessContext,
    args: Vec<Object>,
) -> Object {
    let time = args[0].as_i64().unwrap() as u64;
    thread::sleep(Duration::from_millis(time));
    return Object::I64(time as i64);
}
