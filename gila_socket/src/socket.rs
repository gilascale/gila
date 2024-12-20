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

#[no_mangle]
pub extern "C" fn listen_socket(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    let port = &args[0];
    println!("listen_socket port={:?}", port);
    return Object::I64(12);
}

#[no_mangle]
pub extern "C" fn accept_socket(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    let socket = &args[0];
    println!("accept_socket socket={:?}", socket);
    return Object::I64(12);
}

#[no_mangle]
pub extern "C" fn send_socket(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    let socket = &args[0];
    println!("send_socket socket={:?}", socket);
    return Object::I64(12);
}

#[no_mangle]
pub extern "C" fn close_socket(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    let socket = &args[0];
    println!("closing socket={:?}", socket);
    return Object::I64(12);
}
