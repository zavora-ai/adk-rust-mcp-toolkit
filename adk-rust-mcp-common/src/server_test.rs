//! Unit tests for server builder utilities.

use super::server::{ServerError, shutdown_channel};

#[test]
fn test_server_error_bind_failed_display() {
    let err = ServerError::BindFailed {
        port: 8080,
        message: "Address already in use".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("8080"), "Should contain port number");
    assert!(
        msg.contains("Address already in use"),
        "Should contain error message"
    );
}

#[test]
fn test_server_error_transport_display() {
    let err = ServerError::Transport("Connection reset".to_string());
    let msg = err.to_string();
    assert!(
        msg.contains("Connection reset"),
        "Should contain transport error"
    );
}

#[test]
fn test_server_error_shutdown_display() {
    let err = ServerError::Shutdown;
    let msg = err.to_string();
    assert!(msg.contains("shutdown"), "Should mention shutdown");
}

#[test]
fn test_server_error_io_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: ServerError = io_err.into();
    assert!(matches!(err, ServerError::Io(_)));
}

#[test]
fn test_shutdown_channel() {
    let (tx, rx) = shutdown_channel();

    // Verify we can send a shutdown signal
    assert!(tx.send(()).is_ok());

    // The receiver should have received the signal
    // (we can't easily test this without async, but the types are correct)
    drop(rx);
}

#[tokio::test]
async fn test_shutdown_channel_async() {
    let (tx, rx) = shutdown_channel();

    // Spawn a task that will send the shutdown signal
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let _ = tx.send(());
    });

    // Wait for the shutdown signal
    let result = rx.await;
    assert!(result.is_ok(), "Should receive shutdown signal");
}
