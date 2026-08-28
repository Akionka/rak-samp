use samp_protocol::{
    IncomingPacketDescriptor, IncomingRpcDescriptor, OutgoingPacketDescriptor,
    OutgoingRpcDescriptor,
    packet::common::{AuthenticationRequest, SendAimSync},
    rpc::{incoming::common::ServerMessageRpc, outgoing::chat::SendChat},
};

fn require_incoming_packet<D: IncomingPacketDescriptor>() {}
fn require_outgoing_packet<D: OutgoingPacketDescriptor>() {}
fn require_incoming_rpc<D: IncomingRpcDescriptor>() {}
fn require_outgoing_rpc<D: OutgoingRpcDescriptor>() {}

#[test]
fn directional_descriptors_implement_only_their_directional_bound() {
    require_incoming_packet::<AuthenticationRequest>();
    require_outgoing_packet::<SendAimSync>();
    require_incoming_rpc::<ServerMessageRpc>();
    require_outgoing_rpc::<SendChat>();
}
