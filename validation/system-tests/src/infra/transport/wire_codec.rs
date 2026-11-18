// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::Epoch;
use faction_protocol::transport_message::TransportMessage;

const EPOCH: usize = 8;
const LENGTH: usize = 4;
const SENDER: usize = 8;
const REQUIRED_COUNT: usize = 8;
const MEMBER_COUNT: usize = 4;
const PEER: usize = 8;

// The bytes a message travels as over TCP and gRPC: a frame, its length -- of
// what follows it -- then the sender, a tag and the payload. A ping's, a
// readiness' and an acknowledgement's payload is the sender's epoch; a
// membership's, its epoch, the required count, the number of members, each
// member, then the peer it is addressed to; a probe's and its answer's, a flag and, set, the node it is
// relayed for; an indirect probe's and its answer's, the target.
pub struct WireCodec;

impl WireCodec {
    #[must_use]
    pub fn decode(frame: &[u8]) -> Option<TransportMessage> {
        let length =
            usize::try_from(u32::from_le_bytes(frame.get(..LENGTH)?.try_into().ok()?)).ok()?;
        let body = frame.get(LENGTH..).filter(|body| body.len() == length)?;
        let from = u64::from_le_bytes(body.get(..SENDER)?.try_into().ok()?);
        let payload = body.get(SENDER + 1..)?;
        match body.get(SENDER)? {
            0 => Self::epoch(payload).map(|epoch| TransportMessage::Ping { from, epoch }),
            1 => Self::epoch(payload).map(|epoch| TransportMessage::Ready { from, epoch }),
            2 => Some(TransportMessage::Bootstrapped { from }),
            3 => Self::membership(from, payload),
            4 => Self::epoch(payload)
                .map(|epoch| TransportMessage::MembershipAcknowledged { from, epoch }),
            5 => {
                Self::relayed(payload).map(|relay_for| TransportMessage::Probe { from, relay_for })
            }
            6 => Self::relayed(payload)
                .map(|relay_for| TransportMessage::ProbeAnswer { from, relay_for }),
            7 => Self::peer(payload).map(|target| TransportMessage::IndirectProbe { from, target }),
            8 => Self::peer(payload)
                .map(|target| TransportMessage::IndirectProbeAnswer { from, target }),
            _ => None,
        }
    }

    #[must_use]
    pub fn encode(message: &TransportMessage) -> Vec<u8> {
        let (from, tag, payload): (u64, u8, Vec<u8>) = match message {
            TransportMessage::Ping { from, epoch } => (*from, 0, epoch.to_le_bytes().to_vec()),
            TransportMessage::Ready { from, epoch } => (*from, 1, epoch.to_le_bytes().to_vec()),
            TransportMessage::Bootstrapped { from } => (*from, 2, Vec::new()),
            TransportMessage::Membership {
                from,
                to,
                members,
                required_count,
                epoch,
            } => (
                *from,
                3,
                Self::membership_payload(*to, members, *required_count, *epoch),
            ),
            TransportMessage::MembershipAcknowledged { from, epoch } => {
                (*from, 4, epoch.to_le_bytes().to_vec())
            }
            TransportMessage::Probe { from, relay_for } => {
                (*from, 5, Self::relay_payload(*relay_for))
            }
            TransportMessage::ProbeAnswer { from, relay_for } => {
                (*from, 6, Self::relay_payload(*relay_for))
            }
            TransportMessage::IndirectProbe { from, target } => {
                (*from, 7, target.to_le_bytes().to_vec())
            }
            TransportMessage::IndirectProbeAnswer { from, target } => {
                (*from, 8, target.to_le_bytes().to_vec())
            }
        };
        let body: Vec<u8> = from
            .to_le_bytes()
            .into_iter()
            .chain([tag])
            .chain(payload)
            .collect();
        let length = u32::try_from(body.len()).expect("a frame shorter than 4 GiB");
        length.to_le_bytes().into_iter().chain(body).collect()
    }

    fn epoch(payload: &[u8]) -> Option<Epoch> {
        payload.try_into().ok().map(Epoch::from_le_bytes)
    }

    fn membership(from: u64, framed: &[u8]) -> Option<TransportMessage> {
        let epoch = Epoch::from_le_bytes(framed.get(..EPOCH)?.try_into().ok()?);
        let payload = framed.get(EPOCH..)?;
        let required_count = usize::try_from(u64::from_le_bytes(
            payload.get(..REQUIRED_COUNT)?.try_into().ok()?,
        ))
        .ok()?;
        let member_count = usize::try_from(u32::from_le_bytes(
            payload
                .get(REQUIRED_COUNT..REQUIRED_COUNT + MEMBER_COUNT)?
                .try_into()
                .ok()?,
        ))
        .ok()?;
        let listed = payload.get(REQUIRED_COUNT + MEMBER_COUNT..)?;
        let (members, to) = listed.split_at(listed.len().checked_sub(PEER)?);
        let members = Some(members)
            .filter(|members| members.len() == member_count * PEER)?
            .chunks_exact(PEER)
            .map(|member| u64::from_le_bytes(member.try_into().expect("a member's eight bytes")))
            .collect();
        Some(TransportMessage::Membership {
            from,
            to: Self::peer(to)?,
            members,
            required_count,
            epoch,
        })
    }

    fn membership_payload(
        to: u64,
        members: &[u64],
        required_count: usize,
        epoch: Epoch,
    ) -> Vec<u8> {
        let required_count = u64::try_from(required_count).expect("a count within a u64");
        let member_count = u32::try_from(members.len()).expect("fewer members than 2^32");
        epoch
            .to_le_bytes()
            .into_iter()
            .chain(required_count.to_le_bytes())
            .chain(member_count.to_le_bytes())
            .chain(members.iter().flat_map(|member| member.to_le_bytes()))
            .chain(to.to_le_bytes())
            .collect()
    }

    fn peer(payload: &[u8]) -> Option<u64> {
        payload.try_into().ok().map(u64::from_le_bytes)
    }

    fn relay_payload(relay_for: Option<u64>) -> Vec<u8> {
        relay_for.map_or(vec![0], |peer| {
            [1].into_iter().chain(peer.to_le_bytes()).collect()
        })
    }

    // A flag, then the node a probe is relayed for if the flag is set.
    fn relayed(payload: &[u8]) -> Option<Option<u64>> {
        match payload.split_first()? {
            (0, []) => Some(None),
            (1, peer) => Self::peer(peer).map(Some),
            _ => None,
        }
    }
}
