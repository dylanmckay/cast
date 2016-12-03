use wire;
use json;

/// The version of the CAST protocol we are using.
const PROTOCOL_VERSION: wire::CastMessage_ProtocolVersion = wire::CastMessage_ProtocolVersion::CASTV2_1_0;

/// The namespace a message is send over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Namespace(pub String);

/// A sender/receive ID.
/// Examples:
/// * `receiver-0`
/// * `sender-0`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointName(pub String);

/// A CASTV2 message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message
{
    /// The sender ID of the message.
    pub source: EndpointName,
    pub destination: EndpointName,
    pub namespace: Namespace,
    pub kind: MessageKind,
}

/// A message variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageKind
{
    /// Create a virtual connection.
    Connect,
    /// Close the virtual connection.
    Close,
    /// Ask for a pong message to be sent pack.
    Ping,
    /// Response to a ping message
    Pong,
    /// Get the current receiver status.
    GetStatus,
    /// Tell the Cast device to launch an application.
    Launch {
        /// An application identifier.
        app_id: String,
        /// A request identifier.
        request_id: i64,
    },
    /// Tell the sender about the current receiver status.
    ReceiverStatus {
        status: String,
    },
}

impl Message
{
    /// Creates a message from a wire message.
    pub fn from_wire_message(message: &wire::CastMessage) -> Result<Self, String> {
        let kind = match message.get_payload_type() {
            wire::CastMessage_PayloadType::STRING => {
                let data = json::parse(message.get_payload_utf8()).unwrap();

                match data["type"].as_str().expect("type is not string") {
                    "CONNECT" => MessageKind::Connect,
                    "CLOSE" => MessageKind::Close,
                    "PING" => MessageKind::Ping,
                    "PONG" => MessageKind::Pong,
                    "GET_STATUS" => MessageKind::GetStatus,
                    "LAUNCH" => unimplemented!(),
                    "RECEIVER_STATUS" => {
                        MessageKind::ReceiverStatus {
                            status: message.get_payload_utf8().to_string(),
                        }
                    },
                    _ => return Err(format!("unknown packet type: {}", data["type"])),
                }
            },
            wire::CastMessage_PayloadType::BINARY => {
                return Err("unimplemented binary packet type".to_string());
            },
        };

        Ok(Message {
            source: EndpointName(message.get_source_id().to_owned()),
            destination: EndpointName(message.get_destination_id().to_owned()),
            namespace: Namespace(message.get_namespace().to_owned()),
            kind: kind,
        })
    }

    /// Builds a wire message.
    pub fn as_wire_message(&self) -> wire::CastMessage {
        let mut message = wire::CastMessage::new();

        message.set_protocol_version(PROTOCOL_VERSION);
        message.set_source_id(self.source.0.clone());
        message.set_destination_id(self.destination.0.clone());
        message.set_namespace(self.namespace.0.clone());

        match self.kind {
            MessageKind::Connect => {
                message.set_payload_type(wire::CastMessage_PayloadType::STRING);
                message.set_payload_utf8("{ \"type\": \"CONNECT\" }".to_owned());
            },
            MessageKind::Close => {
                message.set_payload_type(wire::CastMessage_PayloadType::STRING);
                message.set_payload_utf8("{ \"type\": \"CLOSE\" }".to_owned());
            },
            MessageKind::Ping => {
                message.set_payload_type(wire::CastMessage_PayloadType::STRING);
                message.set_payload_utf8("{\"type\":\"PING\"}".to_owned());
            },
            MessageKind::Pong => {
                message.set_payload_type(wire::CastMessage_PayloadType::STRING);
                message.set_payload_utf8("{\"type\":\"PONG\"}".to_owned());
            },
            MessageKind::GetStatus => {
                message.set_payload_type(wire::CastMessage_PayloadType::STRING);
                message.set_payload_utf8("{ \"type\": \"GET_STATUS\" }".to_owned());
            },
            MessageKind::Launch { ref app_id, request_id } => {
                message.set_payload_type(wire::CastMessage_PayloadType::STRING);
                message.set_payload_utf8(json::stringify(object! {
                    "type" => "LAUNCH",
                    "appId" => &app_id[..],
                    "requestId" => request_id
                }));
            },
            MessageKind::ReceiverStatus { .. } => unimplemented!(),
        }

        message
    }
}

impl EndpointName
{
    pub fn is_broadcast(&self) -> bool { self.0 == "*" }
}