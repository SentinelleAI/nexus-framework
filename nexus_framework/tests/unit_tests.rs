#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use nexus_framework::config::NexusConfig;
    use nexus_framework::container::DependencyContainer;
    use nexus_framework::error::{AppError, ErrorResponse};

    // ─── DI Container Tests ──────────────────────────────────────────────────

    #[test]
    fn container_build_returns_valid_container() {
        let container = DependencyContainer::build();
        // No services registered in test context, but container should build
        assert_eq!(container.service_count(), 0);
    }

    #[test]
    fn container_has_returns_false_for_unregistered_type() {
        let container = DependencyContainer::build();
        assert!(!container.has::<String>());
    }

    #[test]
    fn container_try_get_returns_none_for_unregistered_type() {
        let container = DependencyContainer::build();
        assert!(container.try_get::<String>().is_none());
    }

    #[test]
    #[should_panic(expected = "not found in container")]
    fn container_get_panics_for_unregistered_type() {
        let container = DependencyContainer::build();
        let _: std::sync::Arc<String> = container.get();
    }

    #[test]
    fn container_debug_impl() {
        let container = DependencyContainer::build();
        let debug_str = format!("{:?}", container);
        assert!(debug_str.contains("DependencyContainer"));
        assert!(debug_str.contains("service_count"));
    }

    #[test]
    fn container_clone() {
        let container = DependencyContainer::build();
        let cloned = container.clone();
        assert_eq!(container.service_count(), cloned.service_count());
    }

    // ─── Error Handling Tests ────────────────────────────────────────────────

    #[test]
    fn app_error_bad_request() {
        let error = AppError::bad_request("Invalid input");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Invalid input");
        assert!(error.details.is_none());
    }

    #[test]
    fn app_error_unauthorized() {
        let error = AppError::unauthorized("Not authenticated");
        assert_eq!(error.status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn app_error_forbidden() {
        let error = AppError::forbidden("Access denied");
        assert_eq!(error.status, StatusCode::FORBIDDEN);
    }

    #[test]
    fn app_error_not_found() {
        let error = AppError::not_found("Resource missing");
        assert_eq!(error.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn app_error_conflict() {
        let error = AppError::conflict("Already exists");
        assert_eq!(error.status, StatusCode::CONFLICT);
    }

    #[test]
    fn app_error_unprocessable() {
        let error = AppError::unprocessable("Validation failed");
        assert_eq!(error.status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn app_error_too_many_requests() {
        let error = AppError::too_many_requests("Rate limited");
        assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn app_error_internal() {
        let error = AppError::internal("Something went wrong");
        assert_eq!(error.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn app_error_service_unavailable() {
        let error = AppError::service_unavailable("Try later");
        assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn app_error_gateway_timeout() {
        let error = AppError::gateway_timeout("Upstream timeout");
        assert_eq!(error.status, StatusCode::GATEWAY_TIMEOUT);
    }

    #[test]
    fn app_error_custom_status() {
        let error = AppError::new(StatusCode::IM_A_TEAPOT, "I'm a teapot");
        assert_eq!(error.status, StatusCode::IM_A_TEAPOT);
        assert_eq!(error.message, "I'm a teapot");
    }

    #[test]
    fn app_error_with_details() {
        let error = AppError::bad_request("Validation failed").with_details(serde_json::json!({
            "field": "email",
            "reason": "invalid"
        }));
        assert!(error.details.is_some());
        let details = error.details.unwrap();
        assert_eq!(details["field"], "email");
        assert_eq!(details["reason"], "invalid");
    }

    #[test]
    fn app_error_display() {
        let error = AppError::bad_request("test message");
        let display = format!("{}", error);
        assert!(display.contains("test message"));
        assert!(display.contains("400"));
    }

    #[test]
    fn app_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err = AppError::from(io_err);
        assert_eq!(app_err.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn error_response_serialization() {
        let response = ErrorResponse {
            status: 404,
            error: "Not Found".to_string(),
            message: "Resource not found".to_string(),
            details: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":404"));
        assert!(json.contains("\"Not Found\""));
        // details should be skipped when None
        assert!(!json.contains("\"details\""));
    }

    #[test]
    fn error_response_with_details_serialization() {
        let response = ErrorResponse {
            status: 400,
            error: "Bad Request".to_string(),
            message: "Validation failed".to_string(),
            details: Some(serde_json::json!({"field": "email"})),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"details\""));
        assert!(json.contains("\"email\""));
    }

    // ─── Configuration Tests ─────────────────────────────────────────────────

    #[test]
    fn config_default_values() {
        let config = NexusConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.app.name, "nexus-app");
    }

    #[test]
    fn config_load_with_defaults() {
        // Without nexus.toml, should use defaults
        let config = NexusConfig::load();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn config_get_returns_none_for_missing_key() {
        let config = NexusConfig::load();
        let value: Option<String> = config.get("nonexistent.key");
        assert!(value.is_none());
    }

    #[test]
    fn config_get_returns_built_in_key() {
        let config = NexusConfig::load();
        let host: Option<String> = config.get("server.host");
        assert_eq!(host, Some("0.0.0.0".to_string()));
    }

    #[test]
    fn config_debug_impl() {
        let config = NexusConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("NexusConfig"));
    }

    #[test]
    fn config_clone() {
        let config = NexusConfig::load();
        let cloned = config.clone();
        assert_eq!(config.server.port, cloned.server.port);
    }
}
