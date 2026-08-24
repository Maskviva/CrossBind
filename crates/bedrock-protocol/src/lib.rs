pub mod base;
pub mod connection;
pub mod direction;
pub mod manager;
pub mod mapping;
pub mod packet_ids;
pub mod pipeline;
pub mod rewriter;
pub mod sound_events;
pub mod steps;
pub mod translator;
pub mod versions;

use std::sync::Arc;

pub use connection::ConnState;
pub use direction::Direction;
pub use manager::{Chain, Registry};
pub use mapping::{IdShift, MappingData};
pub use pipeline::{translate, Outcome, Translation};
pub use steps::set_score_v2168::describe_layout as describe_set_score_layout;
pub use translator::{Handler, Translator};
pub use versions::Version;

pub fn build_registry(server_protocol: u32) -> Registry {
    Registry::build(
        server_protocol,
        vec![Arc::new(base::create(server_protocol))],
        steps::all(),
    )
}

pub fn describe_support(registry: &Registry) -> String {
    let clients: Vec<String> = registry
        .supported_clients()
        .into_iter()
        .filter(|protocol| *protocol != registry.server_protocol())
        .map(versions::describe)
        .collect();
    if clients.is_empty() {
        return format!(
            "server is {}; no other versions can be translated",
            versions::describe(registry.server_protocol())
        );
    }
    format!(
        "server is {}; also accepting {}",
        versions::describe(registry.server_protocol()),
        clients.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_supported_server_reaches_every_supported_client() {
        for server in versions::TRANSLATABLE {
            let registry = build_registry(*server);
            for client in versions::TRANSLATABLE {
                assert!(
                    registry.chain(*client).is_some(),
                    "server {server} cannot serve client {client}"
                );
            }
        }
    }

    #[test]
    fn base_step_runs_even_for_a_native_client() {
        let registry = build_registry(944);
        assert_eq!(registry.base().len(), 1);
        assert!(registry.chain(944).unwrap().is_empty());
    }

    #[test]
    fn login_protocol_is_rewritten_to_the_server_version() {
        let registry = build_registry(944);
        let mut state = ConnState::new(944);

        let body = 975i32.to_be_bytes();
        let result = translate(
            &registry,
            &mut state,
            Direction::Serverbound,
            packet_ids::ids::REQUEST_NETWORK_SETTINGS,
            &body,
        );

        assert_eq!(state.client_protocol, 975);
        match result.outcome {
            Outcome::Rewritten(bytes) => {
                assert_eq!(bytes, 944i32.to_be_bytes().to_vec());
            }
            other => panic!("expected a rewrite, got {other:?}"),
        }
    }

    #[test]
    fn unknown_packets_pass_through_untouched() {
        let registry = build_registry(944);
        let mut state = ConnState::new(944);
        state.client_protocol = 975;
        let body = [1u8, 2, 3, 4, 5];
        let result = translate(&registry, &mut state, Direction::Clientbound, 200, &body);
        assert_eq!(result.outcome, Outcome::Unchanged);
    }

    #[test]
    fn an_unreachable_client_warns_once_then_stays_quiet() {
        let registry = build_registry(944);
        let mut state = ConnState::new(944);
        state.client_protocol = 729;

        let first = translate(&registry, &mut state, Direction::Clientbound, 200, &[0u8]);
        assert_eq!(warnings(&first), 1);

        let second = translate(&registry, &mut state, Direction::Clientbound, 200, &[0u8]);
        assert_eq!(warnings(&second), 0);
    }

    #[test]
    fn a_malformed_packet_is_dropped_instead_of_forwarded() {
        let registry = build_registry(944);
        let mut state = ConnState::new(944);
        state.client_protocol = 975;

        let body = [0x01u8, 0xFF];
        let result = translate(
            &registry,
            &mut state,
            Direction::Clientbound,
            packet_ids::ids::SET_ACTOR_DATA,
            &body,
        );
        assert_eq!(result.outcome, Outcome::Drop);
        assert_eq!(warnings(&result), 1);

        let again = translate(
            &registry,
            &mut state,
            Direction::Clientbound,
            packet_ids::ids::SET_ACTOR_DATA,
            &body,
        );
        assert_eq!(again.outcome, Outcome::Drop);
        assert_eq!(warnings(&again), 0, "the same failure repeats silently");
    }

    #[test]
    fn a_malformed_base_step_packet_still_forwards() {
        let registry = build_registry(944);
        let mut state = ConnState::new(944);

        let result = translate(
            &registry,
            &mut state,
            Direction::Serverbound,
            packet_ids::ids::REQUEST_NETWORK_SETTINGS,
            &[0x00u8, 0x01],
        );

        assert_eq!(
            result.outcome,
            Outcome::Unchanged,
            "dropping the handshake would replace a version-mismatch screen \
             with a connection that hangs"
        );
    }

    fn warnings(result: &crate::pipeline::Translation) -> usize {
        result
            .notices
            .iter()
            .filter(|n| !n.starts_with("trace "))
            .count()
    }
}
