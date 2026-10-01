//! The wire protocol for the gateway's network API: newline-delimited
//! JSON, one request per line, one response per line. Used identically
//! by the Unix-socket and (future) WebSocket transports -- `server.rs`
//! dispatches a parsed `Request` the same way regardless of which
//! transport it arrived on.

use serde::{Deserialize, Serialize};

use crate::backend::Command;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Request {
    Arm,
    Disarm,
    Status,
    Feedback {
        joint: String,
    },
    Command {
        joint: String,
        #[serde(flatten)]
        command: Command,
    },
    TripLog {
        #[serde(default)]
        limit: Option<usize>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Response {
    pub fn ok() -> Self {
        Response {
            ok: true,
            error: None,
            data: None,
        }
    }

    pub fn ok_with(data: serde_json::Value) -> Self {
        Response {
            ok: true,
            error: None,
            data: Some(data),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Response {
            ok: false,
            error: Some(message.into()),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_arm_request() {
        let req: Request = serde_json::from_str(r#"{"cmd":"arm"}"#).unwrap();
        assert!(matches!(req, Request::Arm));
    }

    #[test]
    fn parses_command_request_with_flattened_fields() {
        let req: Request =
            serde_json::from_str(r#"{"cmd":"command","joint":"knee","torque_nm":1.5}"#).unwrap();
        match req {
            Request::Command { joint, command } => {
                assert_eq!(joint, "knee");
                assert_eq!(command.torque_nm, Some(1.5));
                assert_eq!(command.position_rad, None);
            }
            other => panic!("expected Command, got {other:?}"),
        }
    }

    #[test]
    fn parses_trip_log_request_with_missing_limit_as_none() {
        let req: Request = serde_json::from_str(r#"{"cmd":"trip_log"}"#).unwrap();
        match req {
            Request::TripLog { limit } => assert_eq!(limit, None),
            other => panic!("expected TripLog, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_cmd() {
        let result: Result<Request, _> = serde_json::from_str(r#"{"cmd":"not_a_real_cmd"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn response_ok_serializes_without_null_fields() {
        let json = serde_json::to_string(&Response::ok()).unwrap();
        assert_eq!(json, r#"{"ok":true}"#);
    }

    #[test]
    fn response_err_includes_message() {
        let json = serde_json::to_string(&Response::err("not armed")).unwrap();
        assert_eq!(json, r#"{"ok":false,"error":"not armed"}"#);
    }
}
