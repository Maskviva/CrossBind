use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::translator::Translator;

#[derive(Clone, Default)]
pub struct Chain {
    pub serverbound: Vec<Arc<Translator>>,
    pub clientbound: Vec<Arc<Translator>>,
}

impl Chain {
    pub fn is_empty(&self) -> bool {
        self.serverbound.is_empty() && self.clientbound.is_empty()
    }

    pub fn len(&self) -> usize {
        self.serverbound.len()
    }

    pub fn describe(&self) -> String {
        if self.serverbound.is_empty() {
            return "direct (no translation)".to_owned();
        }
        self.serverbound
            .iter()
            .map(|step| step.name)
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

pub struct Registry {
    server_protocol: u32,
    base: Vec<Arc<Translator>>,
    chains: HashMap<u32, Chain>,
}

impl Registry {
    pub fn build(
        server_protocol: u32,
        base: Vec<Arc<Translator>>,
        steps: Vec<Arc<Translator>>,
    ) -> Registry {
        let mut incoming: HashMap<u32, Vec<Arc<Translator>>> = HashMap::new();
        for step in &steps {
            incoming
                .entry(step.server_protocol)
                .or_default()
                .push(Arc::clone(step));
        }

        let mut chains: HashMap<u32, Chain> = HashMap::new();
        chains.insert(server_protocol, Chain::default());

        let mut seen: HashSet<u32> = HashSet::new();
        seen.insert(server_protocol);
        let mut queue: VecDeque<u32> = VecDeque::new();
        queue.push_back(server_protocol);

        while let Some(current) = queue.pop_front() {
            let Some(candidates) = incoming.get(&current) else {
                continue;
            };
            for step in candidates {
                let next = step.client_protocol;
                if !seen.insert(next) {
                    continue;
                }
                let parent = chains.get(&current).cloned().unwrap_or_default();
                let mut serverbound = Vec::with_capacity(parent.serverbound.len() + 1);
                serverbound.push(Arc::clone(step));
                serverbound.extend(parent.serverbound.iter().cloned());

                let mut clientbound = serverbound.clone();
                clientbound.reverse();

                chains.insert(
                    next,
                    Chain {
                        serverbound,
                        clientbound,
                    },
                );
                queue.push_back(next);
            }
        }

        Registry {
            server_protocol,
            base,
            chains,
        }
    }

    pub fn server_protocol(&self) -> u32 {
        self.server_protocol
    }

    pub fn base(&self) -> &[Arc<Translator>] {
        &self.base
    }

    pub fn chain(&self, client_protocol: u32) -> Option<&Chain> {
        self.chains.get(&client_protocol)
    }

    pub fn supported_clients(&self) -> Vec<u32> {
        let mut out: Vec<u32> = self.chains.keys().copied().collect();
        out.sort_unstable();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(name: &'static str, server: u32, client: u32) -> Arc<Translator> {
        Arc::new(Translator::new(name, server, client))
    }

    fn ladder(server_protocol: u32) -> Registry {
        let versions = [859u32, 860, 898, 924, 944, 975];
        let mut steps = Vec::new();
        for pair in versions.windows(2) {
            let (lo, hi) = (pair[0], pair[1]);
            steps.push(step("down", lo, hi));
            steps.push(step("up", hi, lo));
        }
        Registry::build(server_protocol, Vec::new(), steps)
    }

    #[test]
    fn native_client_gets_an_empty_chain() {
        let reg = ladder(944);
        let chain = reg.chain(944).expect("server's own protocol");
        assert!(chain.is_empty());
    }

    #[test]
    fn every_version_reaches_every_server() {
        for server in [859u32, 860, 898, 924, 944, 975] {
            let reg = ladder(server);
            for client in [859u32, 860, 898, 924, 944, 975] {
                assert!(
                    reg.chain(client).is_some(),
                    "client {client} cannot reach server {server}"
                );
            }
        }
    }

    #[test]
    fn chain_length_is_the_number_of_hops() {
        let reg = ladder(860);
        assert_eq!(reg.chain(975).unwrap().len(), 4);
        assert_eq!(reg.chain(898).unwrap().len(), 1);
        assert_eq!(reg.chain(859).unwrap().len(), 1);
    }

    #[test]
    fn clientbound_is_the_reverse_of_serverbound() {
        let reg = ladder(860);
        let chain = reg.chain(975).unwrap();
        let up: Vec<u32> = chain.serverbound.iter().map(|s| s.client_protocol).collect();
        let down: Vec<u32> = chain
            .clientbound
            .iter()
            .rev()
            .map(|s| s.client_protocol)
            .collect();
        assert_eq!(up, down);
    }

    #[test]
    fn serverbound_chain_walks_toward_the_server() {
        let reg = ladder(860);
        let chain = reg.chain(975).unwrap();
        assert_eq!(chain.serverbound[0].client_protocol, 975);
        for pair in chain.serverbound.windows(2) {
            assert_eq!(
                pair[0].server_protocol, pair[1].client_protocol,
                "chain is not contiguous"
            );
        }
        assert_eq!(chain.serverbound.last().unwrap().server_protocol, 860);
    }

    #[test]
    fn unreachable_version_has_no_chain() {
        let reg = ladder(860);
        assert!(reg.chain(729).is_none());
    }

    #[test]
    fn breadth_first_picks_the_shortest_route() {
        let mut steps = vec![step("shortcut", 860, 975)];
        for pair in [859u32, 860, 898, 924, 944, 975].windows(2) {
            steps.push(step("down", pair[0], pair[1]));
            steps.push(step("up", pair[1], pair[0]));
        }
        let reg = Registry::build(860, Vec::new(), steps);
        assert_eq!(reg.chain(975).unwrap().len(), 1);
    }
}
