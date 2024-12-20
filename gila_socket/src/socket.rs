use gila::execution::Object;
use gila::execution::ProcessContext;
use gila::execution::SharedExecutionContext;

#[no_mangle]
pub extern "C" fn create_socket(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    let port = &args[0];
    println!("create_socket port={:?}", port);
    return Object::I64(12);
}
