use gila::execution::Object;
use gila::execution::ProcessContext;
use gila::execution::SharedExecutionContext;

#[no_mangle]
pub fn create_socket(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    Object::I64(12)
}
