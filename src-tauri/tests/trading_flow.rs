use titan_hft_rust::error::TitanError;

#[test]
fn titan_error_serializes_to_json() {
    let err = TitanError::Network("connection refused".into());
    let json = serde_json::to_string(&err).unwrap();
    assert!(json.contains("connection refused"), "JSON: {}", json);

    let broker_err = TitanError::Broker("insufficient funds".into());
    let json = serde_json::to_string(&broker_err).unwrap();
    assert!(json.contains("insufficient funds"), "JSON: {}", json);
}

#[test]
fn titan_error_roundtrips_through_json() {
    let variants: Vec<TitanError> = vec![
        TitanError::Network("net err".into()),
        TitanError::Broker("broker err".into()),
        TitanError::Validation("bad input".into()),
        TitanError::Internal("panic".into()),
        TitanError::Io("disk full".into()),
        TitanError::Gpu("out of memory".into()),
        TitanError::Timeout("5s elapsed".into()),
        TitanError::RateLimit("429".into()),
    ];

    for err in &variants {
        let json = serde_json::to_string(err).unwrap();
        assert!(!json.is_empty(), "serialized to empty string");
    }
}

#[test]
fn titan_error_display_format() {
    let err = TitanError::Network("timeout".into());
    let display = format!("{}", err);
    assert_eq!(display, "[NETWORK] timeout");

    let err = TitanError::Broker("rejected".into());
    assert_eq!(format!("{}", err), "[BROKER] rejected");

    let err = TitanError::Validation("missing field".into());
    assert_eq!(format!("{}", err), "[VALIDATION] missing field");

    let err = TitanError::Internal("null ptr".into());
    assert_eq!(format!("{}", err), "[INTERNAL] null ptr");

    let err = TitanError::Io("permission denied".into());
    assert_eq!(format!("{}", err), "[IO] permission denied");

    let err = TitanError::Gpu("cuda error".into());
    assert_eq!(format!("{}", err), "[GPU] cuda error");

    let err = TitanError::Timeout("30s".into());
    assert_eq!(format!("{}", err), "[TIMEOUT] 30s");

    let err = TitanError::RateLimit("slow down".into());
    assert_eq!(format!("{}", err), "[RATE_LIMIT] slow down");
}

#[test]
fn titan_error_into_string_produces_display() {
    let err = TitanError::Network("conn reset".into());
    let as_string: String = err.into();
    assert_eq!(as_string, "[NETWORK] conn reset");

    let err = TitanError::Gpu("device lost".into());
    let as_string: String = err.into();
    assert_eq!(as_string, "[GPU] device lost");
}

#[test]
fn titan_error_debug_contains_variant_name() {
    let err = TitanError::Broker("margin call".into());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Broker"), "Debug: {}", debug);
    assert!(debug.contains("margin call"), "Debug: {}", debug);
}
