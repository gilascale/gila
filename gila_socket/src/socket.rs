use gila::execution::Object;
use gila::execution::ProcessContext;
use gila::execution::SharedExecutionContext;
use libc::*;
use windows::Win32::Networking::WinSock::{
    WSACleanup, WSAGetLastError, WSAStartup, WSADATA, WSADESCRIPTION_LEN, WSASYS_STATUS_LEN,
};

const AF_INET: c_int = 2;
const SOCK_STREAM: c_int = 1;
const INADDR_ANY: c_int = 0;

fn makeword(low: u8, high: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

fn htons(host_short: u16) -> u16 {
    host_short.to_be()
}

#[no_mangle]
pub extern "C" fn initialise(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &ProcessContext,
    args: Vec<Object>,
) -> Object {
    println!("initialing sockets...");

    // Prepare the WSADATA struct
    let mut wsa_data = WSADATA::default();

    let result = unsafe { WSAStartup(makeword(2, 2), &mut wsa_data) };

    if result != 0 {
        // Handle error
        let error_code = unsafe { WSAGetLastError() };
        panic!("WSAStartup failed with error code: {:?}", error_code);
    }

    return Object::I64(0);
}

#[no_mangle]
pub extern "C" fn deinit(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &ProcessContext,
    args: Vec<Object>,
) -> Object {
    println!("deinit sockets...");

    unsafe {
        WSACleanup();
    }

    return Object::I64(0);
}

#[no_mangle]
pub extern "C" fn create_socket(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    unsafe {
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct sockaddr_in {
    sin_family: u16,   // Address family (AF_INET)
    sin_port: u16,     // Port number
    sin_addr: u32,     // IPv4 address
    sin_zero: [u8; 8], // Padding to match the size of `sockaddr`
}

#[no_mangle]
pub extern "C" fn listen_socket(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    let socket = args[0].as_i64().unwrap() as c_int;
    let port = args[0].as_i64().unwrap() as c_int;

    // Create and populate the sockaddr_in struct
    let mut addr = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: htons(port as u16),
        sin_addr: INADDR_ANY as u32,
        sin_zero: [0; 8],
    };

    unsafe {
        // Bind the socket to the port
        if bind(
            std::mem::size_of_val(&addr) as usize,
            &addr as *const _ as *const sockaddr,
            socket,
        ) == -1
        {
            let err_code = std::io::Error::last_os_error().raw_os_error().unwrap();
            panic!("Failed to bind socket, errno = {:?}", err_code);
        }

        // Start listening on the socket
        if listen(socket.try_into().unwrap(), 5) == -1 {
            let err_code = std::io::Error::last_os_error().raw_os_error().unwrap();
            panic!("Failed to listen on socket, errno = {:?}", err_code);
        }
    }
    println!("Socket is listening on port {}", port);
    return Object::I64(0);
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
