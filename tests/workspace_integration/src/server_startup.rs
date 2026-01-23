//! Server startup integration tests.
//!
//! Tests that each MCP server can be instantiated and provides correct server info.
//! These tests verify Requirements 3.7, 3.8 - tool and resource registration.

use adk_rust_mcp_common::Config;
use rmcp::ServerHandler;

/// Test configuration for integration tests.
fn test_config() -> Config {
    Config {
        project_id: "test-project".to_string(),
        location: "us-central1".to_string(),
        gcs_bucket: None,
        port: 8080,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adk_rust_mcp_avtool::AVToolServer;
    use adk_rust_mcp_image::ImageServer;
    use adk_rust_mcp_multimodal::MultimodalServer;
    use adk_rust_mcp_music::MusicServer;
    use adk_rust_mcp_speech::SpeechServer;
    use adk_rust_mcp_video::VideoServer;

    /// Test that ImageServer can be created and provides server info.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_image_server_startup() {
        let config = test_config();
        let server = ImageServer::new(config);
        let info = server.get_info();

        assert!(info.instructions.is_some());
        let instructions = info.instructions.as_ref().unwrap().to_lowercase();
        assert!(
            instructions.contains("image"),
            "Server instructions should mention 'image'"
        );
    }

    /// Test that VideoServer can be created and provides server info.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_video_server_startup() {
        let config = test_config();
        let server = VideoServer::new(config);
        let info = server.get_info();

        assert!(info.instructions.is_some());
    }

    /// Test that MusicServer can be created and provides server info.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_music_server_startup() {
        let config = test_config();
        let server = MusicServer::new(config);
        let info = server.get_info();

        assert!(info.instructions.is_some());
    }

    /// Test that SpeechServer can be created and provides server info.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_speech_server_startup() {
        let config = test_config();
        let server = SpeechServer::new(config);
        let info = server.get_info();

        assert!(info.instructions.is_some());
    }

    /// Test that MultimodalServer can be created and provides server info.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_multimodal_server_startup() {
        let config = test_config();
        let server = MultimodalServer::new(config);
        let info = server.get_info();

        assert!(info.instructions.is_some());
    }

    /// Test that AVToolServer can be created and provides server info.
    /// **Validates: Requirements 3.7, 3.8**
    #[test]
    fn test_avtool_server_startup() {
        let config = test_config();
        let server = AVToolServer::new(config);
        let info = server.get_info();

        assert!(info.instructions.is_some());
    }

    /// Test that all servers have tools capability enabled.
    /// **Validates: Requirements 3.7**
    #[test]
    fn test_all_servers_have_tools_capability() {
        let config = test_config();

        // Image server
        let server = ImageServer::new(config.clone());
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());

        // Video server
        let server = VideoServer::new(config.clone());
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());

        // Music server
        let server = MusicServer::new(config.clone());
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());

        // Speech server
        let server = SpeechServer::new(config.clone());
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());

        // Multimodal server
        let server = MultimodalServer::new(config.clone());
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());

        // AVTool server
        let server = AVToolServer::new(config);
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());
    }

    /// Test that servers with resources have resources capability enabled.
    /// **Validates: Requirements 3.8**
    #[test]
    fn test_servers_with_resources_capability() {
        let config = test_config();

        // Image server has resources
        let server = ImageServer::new(config.clone());
        let info = server.get_info();
        assert!(info.capabilities.resources.is_some());

        // Video server has resources
        let server = VideoServer::new(config.clone());
        let info = server.get_info();
        assert!(info.capabilities.resources.is_some());

        // Multimodal server has resources
        let server = MultimodalServer::new(config);
        let info = server.get_info();
        assert!(info.capabilities.resources.is_some());
    }
}
