use gila::execution::Object;
use gila::execution::ProcessContext;
use gila::execution::SharedExecutionContext;
use libc::*;

const AF_INET: c_int = 2;
const SOCK_STREAM: c_int = 1;

#[no_mangle]
pub extern "C" fn create_socket(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    unsafe {
        let port = &args[0].as_i64();

        // Initialize Winsock
        // let mut wsa_data: WSADATA = mem::zeroed();
        // let result = WSAStartup(0x202, &mut wsa_data); // 0x202 means version 2.2 of Winsock
        // if result != 0 {
        //     return Err(format!("WSAStartup failed with error code: {}", result));
        // }

        let socket_fd = socket(AF_INET, SOCK_STREAM, 0);
        if socket_fd as i64 == -1 {
            // Check the errno for more detailed error information
            let err_code = std::io::Error::last_os_error().raw_os_error().unwrap();
            panic!("Failed to create socket, errno = {:?}", err_code);
        }

        println!("create_socket socket_fd={:?}", socket_fd);
        return Object::I64(socket_fd as i64);
    }
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
