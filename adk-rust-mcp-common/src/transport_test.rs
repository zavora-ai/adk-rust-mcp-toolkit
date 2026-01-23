//! Unit tests for transport configuration.

use super::transport::{Transport, TransportArgs, TransportMode};

#[test]
fn test_transport_default_is_stdio() {
    let transport = Transport::default();
    assert!(transport.is_stdio());
    assert!(!transport.is_http());
    assert!(!transport.is_sse());
    assert_eq!(transport.port(), None);
}

#[test]
fn test_transport_stdio_constructor() {
    let transport = Transport::stdio();
    assert!(transport.is_stdio());
    assert_eq!(transport.to_string(), "stdio");
}

#[test]
fn test_transport_http_constructor() {
    let transport = Transport::http(3000);
    assert!(transport.is_http());
    assert!(!transport.is_stdio());
    assert!(!transport.is_sse());
    assert_eq!(transport.port(), Some(3000));
    assert_eq!(transport.to_string(), "http (port 3000)");
}

#[test]
fn test_transport_sse_constructor() {
    let transport = Transport::sse(8080);
    assert!(transport.is_sse());
    assert!(!transport.is_stdio());
    assert!(!transport.is_http());
    assert_eq!(transport.port(), Some(8080));
    assert_eq!(transport.to_string(), "sse (port 8080)");
}

#[test]
fn test_transport_args_default() {
    let args = TransportArgs::default();
    assert_eq!(args.transport, TransportMode::Stdio);
    assert_eq!(args.port, 8080);
}

#[test]
fn test_transport_args_into_transport_stdio() {
    let args = TransportArgs {
        transport: TransportMode::Stdio,
        port: 9000,
    };
    let transport = args.into_transport();
    assert!(transport.is_stdio());
    // Port is ignored for stdio
    assert_eq!(transport.port(), None);
}

#[test]
fn test_transport_args_into_transport_http() {
    let args = TransportArgs {
        transport: TransportMode::Http,
        port: 3000,
    };
    let transport = args.into_transport();
    assert!(transport.is_http());
    assert_eq!(transport.port(), Some(3000));
}

#[test]
fn test_transport_args_into_transport_sse() {
    let args = TransportArgs {
        transport: TransportMode::Sse,
        port: 4000,
    };
    let transport = args.into_transport();
    assert!(transport.is_sse());
    assert_eq!(transport.port(), Some(4000));
}

#[test]
fn test_transport_equality() {
    assert_eq!(Transport::Stdio, Transport::Stdio);
    assert_eq!(Transport::Http { port: 8080 }, Transport::Http { port: 8080 });
    assert_eq!(Transport::Sse { port: 8080 }, Transport::Sse { port: 8080 });

    assert_ne!(Transport::Stdio, Transport::Http { port: 8080 });
    assert_ne!(Transport::Http { port: 8080 }, Transport::Sse { port: 8080 });
    assert_ne!(Transport::Http { port: 8080 }, Transport::Http { port: 9000 });
}

#[test]
fn test_transport_mode_default() {
    let mode = TransportMode::default();
    assert_eq!(mode, TransportMode::Stdio);
}

#[test]
fn test_transport_display() {
    assert_eq!(Transport::Stdio.to_string(), "stdio");
    assert_eq!(Transport::Http { port: 8080 }.to_string(), "http (port 8080)");
    assert_eq!(Transport::Sse { port: 3000 }.to_string(), "sse (port 3000)");
}

// Tests for HTTP port binding (Requirement 3.5)
#[test]
fn test_http_transport_with_various_ports() {
    // Test common ports
    for port in [80, 443, 3000, 8080, 8443, 9000] {
        let transport = Transport::http(port);
        assert!(transport.is_http());
        assert_eq!(transport.port(), Some(port));
    }
}

// Tests for SSE endpoint (Requirement 3.6)
#[test]
fn test_sse_transport_with_various_ports() {
    // Test common ports
    for port in [80, 443, 3000, 8080, 8443, 9000] {
        let transport = Transport::sse(port);
        assert!(transport.is_sse());
        assert_eq!(transport.port(), Some(port));
    }
}

// Test that stdio is the default (Requirement 3.2)
#[test]
fn test_stdio_is_default_transport_mode() {
    // Default TransportArgs should use stdio
    let args = TransportArgs::default();
    let transport = args.into_transport();
    assert!(transport.is_stdio(), "Default transport should be stdio");
}

// Test transport cloning
#[test]
fn test_transport_clone() {
    let original = Transport::Http { port: 8080 };
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// Test transport copy
#[test]
fn test_transport_copy() {
    let original = Transport::Sse { port: 3000 };
    let copied: Transport = original; // Copy
    assert_eq!(original, copied);
}

// Test TransportMode cloning
#[test]
fn test_transport_mode_clone() {
    let original = TransportMode::Http;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// Test TransportArgs cloning
#[test]
fn test_transport_args_clone() {
    let original = TransportArgs {
        transport: TransportMode::Http,
        port: 9000,
    };
    let cloned = original.clone();
    assert_eq!(cloned.transport, TransportMode::Http);
    assert_eq!(cloned.port, 9000);
}
