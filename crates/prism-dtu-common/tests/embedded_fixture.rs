#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Behaviour of the compile-time-embedded fixture parsers (#285, #286).
//!
//! The distinction that matters to callers: `embedded_fixture` panics, matching the
//! route handlers that already unwrapped, while `embedded_fixture_as` returns
//! `anyhow::Result` so a clone constructor can keep propagating with `?`.

use prism_dtu_common::{embedded_fixture, embedded_fixture_as};

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
struct Device {
    id: String,
    name: String,
}

const DEVICES: &str = r#"[{"id":"d-1","name":"plc-01"},{"id":"d-2","name":"hmi-02"}]"#;

#[test]
fn embedded_fixture_as_deserializes_into_a_concrete_type() {
    let devices: Vec<Device> =
        embedded_fixture_as(DEVICES, "devices").expect("valid fixture must deserialize");

    assert_eq!(devices.len(), 2);
    assert_eq!(
        devices[0],
        Device {
            id: "d-1".to_owned(),
            name: "plc-01".to_owned(),
        }
    );
}

#[test]
fn embedded_fixture_as_returns_err_rather_than_panicking_on_malformed_json() {
    let result: anyhow::Result<Vec<Device>> = embedded_fixture_as("{not json", "devices");

    let err = result.expect_err("malformed JSON must be an error, not a panic");
    assert!(
        err.to_string().contains("devices"),
        "error must name the fixture so the caller can tell which one failed, got: {err}"
    );
}

#[test]
fn embedded_fixture_as_returns_err_on_type_mismatch() {
    let result: anyhow::Result<Vec<Device>> = embedded_fixture_as(r#"{"id":"d-1"}"#, "devices");

    assert!(
        result.is_err(),
        "an object where a sequence is expected must be an error"
    );
}

#[test]
fn embedded_fixture_returns_untyped_json() {
    let value = embedded_fixture(DEVICES, "devices");

    assert_eq!(value.as_array().expect("fixture is a JSON array").len(), 2);
    assert_eq!(value[1]["name"], "hmi-02");
}

#[test]
#[should_panic(expected = "embedded fixture 'devices' is not valid JSON")]
fn embedded_fixture_panics_on_malformed_json() {
    let _ = embedded_fixture("{not json", "devices");
}
