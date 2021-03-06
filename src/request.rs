//! jsonrpc request
use serde::de::{Deserialize, Deserializer, Visitor, SeqVisitor, MapVisitor};
use super::{Id, Params, Version, Value};
use super::peek::*;

/// Represents jsonrpc request which is a method call.
#[derive(Debug, PartialEq, Deserialize)]
pub struct MethodCall {
	/// A String specifying the version of the JSON-RPC protocol. 
	/// MUST be exactly "2.0".
	pub jsonrpc: Version,
	/// A String containing the name of the method to be invoked.
	pub method: String,
	/// A Structured value that holds the parameter values to be used 
	/// during the invocation of the method. This member MAY be omitted.
	pub params: Option<Params>,
	/// An identifier established by the Client that MUST contain a String,
	/// Number, or NULL value if included. If it is not included it is assumed 
	/// to be a notification. 
	pub id: Id,
}

/// Represents jsonrpc request which is a notification.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Notification {
	/// A String specifying the version of the JSON-RPC protocol. 
	/// MUST be exactly "2.0".
	pub jsonrpc: Version,
	/// A String containing the name of the method to be invoked.
	pub method: String,
	/// A Structured value that holds the parameter values to be used 
	/// during the invocation of the method. This member MAY be omitted.
	pub params: Option<Params>
}

/// Represents single jsonrpc call.
#[derive(Debug, PartialEq)]
pub enum Call {
	MethodCall(MethodCall),
	Notification(Notification),
	Invalid
}

impl Deserialize for Call {
	fn deserialize<D>(deserializer: &mut D) -> Result<Call, D::Error>
	where D: Deserializer {
		Notification::peek(deserializer).map(Call::Notification)
			.or_else(|_| MethodCall::peek(deserializer).map(Call::MethodCall))
			.or_else(|_| Value::deserialize(deserializer).map(|_| Call::Invalid))
	}
}

/// Represents jsonrpc request.
#[derive(Debug, PartialEq)]
pub enum Request {
	Single(Call),
	Batch(Vec<Call>)
}

impl Deserialize for Request {
	fn deserialize<D>(deserializer: &mut D) -> Result<Request, D::Error>
	where D: Deserializer {
		<Vec<Call> as Peek>::peek(deserializer).map(Request::Batch)
			.or_else(|_| Call::deserialize(deserializer).map(Request::Single))
	}
}

#[test]
fn notification_deserialize() {
	use serde_json;
	use serde_json::Value;

	let s = r#"{"jsonrpc": "2.0", "method": "update", "params": [1,2]}"#;
	let deserialized: Notification = serde_json::from_str(s).unwrap();

	assert_eq!(deserialized, Notification {
		jsonrpc: Version::V2,
		method: "update".to_string(),
		params: Some(Params::Array(vec![Value::U64(1), Value::U64(2)]))
	});

	let s = r#"{"jsonrpc": "2.0", "method": "foobar"}"#;
	let deserialized: Notification = serde_json::from_str(s).unwrap();

	assert_eq!(deserialized, Notification {
		jsonrpc: Version::V2,
		method: "foobar".to_string(),
		params: None
	});

	let s = r#"{"jsonrpc": "2.0", "method": "update", "params": [1,2], "id": 1}"#;
	let deserialized: Result<Notification, _> = serde_json::from_str(s);
	assert!(deserialized.is_err())
}

#[test]
fn call_deserialize_batch() {
	use serde_json;

	let s = r#"[1, {"jsonrpc": "2.0", "method": "update", "params": [1,2], "id": 1},{"jsonrpc": "2.0", "method": "update", "params": [1]}]"#;
	let deserialized: Request = serde_json::from_str(s).unwrap();
	assert_eq!(deserialized, Request::Batch(vec![
		Call::Invalid,
		Call::MethodCall(MethodCall {
			jsonrpc: Version::V2,
			method: "update".to_owned(),
			params: Some(Params::Array(vec![Value::U64(1), Value::U64(2)])),
			id: Id::Num(1)
		}),
		Call::Notification(Notification {
			jsonrpc: Version::V2,
			method: "update".to_owned(),
			params: Some(Params::Array(vec![Value::U64(1)]))
		})
	]))
}
