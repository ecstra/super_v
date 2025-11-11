// System Crates
use std::{
    os::unix::net::{
        UnixListener, 
        UnixStream
    },
    io::{
        Write,
        Read
    },
    fs::remove_file
};

// External Crates
use serde::{
    Serialize, 
    Deserialize
};
use rmp_serde::{Serializer};

// My Crates
use crate::{
    common::{
        IPCServerError,
        SOCKET_PATH
    }, 
    history::ClipboardHistory
};

// ------------------------- IPC Items -------------------------------
/// Represents the commands that IPC Supports
/// 
/// This enum allows for the following commands:
/// * **Promote(usize)** - Command that promotes and item to top of history.
/// * **Delete(usize)** - Command that deletes an item from history given its pos.
/// * **Snapshot** - Command that retrieves the snapshot of the current Clipboard History
/// * **Clear** - Command that clears the entire clipboard History.
#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub enum CmdIPC {
    Promote(usize),
    Delete(usize),
    Snapshot,
    Clear,
}

/// A data structure representing the Response of IPC.
/// 
/// **Contains**:
/// * **history_snapshot** - A snapshot of the current ClipboardHistory from the Clipboard Manager Daemon
/// * **message** - Optional message if there are any errors.
#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub struct IPCResponse { 
    pub history_snapshot: Option<ClipboardHistory>,
    pub message: Option<String>
}

/// A data structure that contains data needed for a payload.
/// 
/// **Contains**:
/// * **buf** - A binary vector of transformed data
/// * **len** - Length of the buf in u8 as bytes
pub struct PayloadData {
    buf: Vec<u8>,
    len: [u8; 4]
}

/// # Payload
/// These are the available Payloads for the IPC Server.
/// 
/// **Available**:
/// * **Cmd(CmdIPC)** - CmdIPC for giving commands
/// * **Resp(IPCResponse)** - IPCResponse that contains a snapshot and a message
#[derive(Debug, Serialize, Deserialize)]
pub enum Payload {
    Cmd(CmdIPC),
    Resp(IPCResponse),
}

impl Payload {
    /// Constructs PayloadData for a given Payload
    fn to_payload(&self) -> PayloadData {
        let mut  buf: Vec<u8> = Vec::new();
        let _ = self.serialize(&mut Serializer::new(&mut buf)).ok();
        let payload_len: [u8; 4] = (buf.len() as u32).to_be_bytes();
        
        PayloadData { 
            buf: buf,
            len: payload_len
        }
    }
}
// -------------------------------------------------------------------

/// Creates and binds a new Unix domain socket listener at SOCKET_PATH.
///
/// # Behavior
/// - If an existing server is already bound to the socket path, it returns an error.
/// - If a stale socket file exists, it removes it before rebinding.
/// - On success, returns a bound `UnixListener`.
///
/// # Errors
/// - Returns `IPCServerError::BindError` if the socket cannot be bound.
/// - Returns `IPCServerError::BindError` if an existing IPC server is detected.
///
/// # Example
/// ```no_run
/// use super_v::services::clipboard_ipc_server::create_bind;
/// let listener = create_bind().expect("Failed to bind IPC server");
/// ```
pub fn create_bind() -> Result<UnixListener, IPCServerError> {
    // Check if we can connect to server.
    // If yes, then server already running and a new server should not start
    let try_conn = create_default_stream();

    let Err(IPCServerError::FileNotFound | IPCServerError::ConnectionError(_)) = try_conn else {
        return Err(IPCServerError::BindError("IPC Server appears to be already running".into()));
    };

    // Remove the old sock file
    let _ = remove_file(SOCKET_PATH);

    // Create a new listener
    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(listener) => {listener},
        Err(err) => {
            return Err(IPCServerError::BindError(String::from(format!("{:?}", err))));
        }
    };

    // Return Listener
    Ok(listener)
}

/// Attempts to connect to the default Unix socket at SOCKET_PATH.
///
/// # Behavior
/// - Returns a connected `UnixStream` if the socket is active.
/// - Handles typical connection failures with custom `IPCServerError` variants.
///
/// # Errors
/// - Returns `IPCServerError::ConnectionError` if connection is refused.
/// - Returns `IPCServerError::FileNotFound` if the socket file is missing.
/// - Returns `IPCServerError::ConnectionError` for any other I/O error.
///
/// # Example
/// ```no_run
/// use super_v::services::clipboard_ipc_server::create_default_stream;
/// let mut stream = create_default_stream().expect("Unable to connect to IPC server");
/// ```
pub fn create_default_stream() -> Result<UnixStream, IPCServerError> {
    match UnixStream::connect(SOCKET_PATH) {
        Ok(stream) => {
            Ok(stream)
        },
        Err(err) => {
            if let Some(err_code) = err.raw_os_error() {
                match err_code {
                    111 => {
                        Err(IPCServerError::ConnectionError("Connection Refused by server.".into()))
                    },
                    2 => {
                        Err(IPCServerError::FileNotFound)
                    }
                    _ => {
                        Err(IPCServerError::ConnectionError(String::from(format!("{:?}", err))))
                    }
                }
            } else {
                Err(IPCServerError::ConnectionError(String::from(format!("{:?}", err))))
            }
        }
    }
}

/// Sends a serialized `Payload` over a connected Unix stream.
///
/// # Behavior
/// - Serializes the `Payload` using MessagePack.
/// - Prepends the payload length (4 bytes, big-endian).
/// - Sends both the length and serialized data through the stream.
/// - Flushes the stream to ensure all data is written.
///
/// # Panics
/// - Panics if the stream fails to write or flush.
///
/// # Example
/// ```no_run
/// use super_v::services::clipboard_ipc_server::{create_default_stream, send_payload, Payload, CmdIPC};
/// let mut stream = create_default_stream().unwrap();
/// send_payload(&mut stream, Payload::Cmd(CmdIPC::Snapshot));
/// ```
pub fn send_payload(stream: &mut UnixStream, item: Payload) {
    // Serialize command
    let payload = item.to_payload();

    // Send len
    // We know the size of the lenght (4).
    // Using that, we can extract the lenght of actual message (x)
    // and read for that len. 
    // This way sending message of changing length works.
    stream.write_all(&payload.len).unwrap();
    
    // Send data
    stream.write_all(&payload.buf).unwrap();

    // Ensure all buffer is written
    stream.flush().unwrap();
}

/// Reads and deserializes a `Payload` from a connected Unix stream.
///
/// # Behavior
/// - Reads the first 4 bytes as a big-endian `u32` payload length.
/// - Reads the following bytes as the serialized payload.
/// - Deserializes the payload into a `Payload` enum instance using MessagePack.
///
/// # Panics
/// - Panics if reading from the stream fails.
/// - Panics if deserialization fails.
///
/// # Example
/// ```no_run
/// use super_v::services::clipboard_ipc_server::{create_default_stream, read_payload};
/// let mut stream = create_default_stream().unwrap();
/// let payload = read_payload(&mut stream);
/// println!("{:?}", payload);
/// ```
pub fn read_payload(stream: &mut UnixStream) -> Payload {
    // Read length of message (u32)
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).unwrap();
    let req_len = u32::from_be_bytes(len_buf) as usize;

    // Read payload
    let mut payload = vec![0u8; req_len];
    stream.read_exact(&mut payload).unwrap();

    // deserialize
    let cmd: Payload = rmp_serde::from_slice(&payload).expect("failed to deserialize");
    cmd
}