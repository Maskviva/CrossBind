use std::collections::{HashMap, HashSet};

use bedrock_codec::{PacketWrapper, Result};

use crate::connection::ConnState;
use crate::direction::Direction;

pub type Handler = Box<dyn Fn(&mut PacketWrapper, &mut ConnState) -> Result<()> + Send + Sync>;

pub struct Translator {
    pub name: &'static str,
    pub server_protocol: u32,
    pub client_protocol: u32,
    clientbound: HashMap<u16, Handler>,
    serverbound: HashMap<u16, Handler>,
    cancel_clientbound: HashSet<u16>,
    cancel_serverbound: HashSet<u16>,
}

impl Translator {
    pub fn new(name: &'static str, server_protocol: u32, client_protocol: u32) -> Translator {
        Translator {
            name,
            server_protocol,
            client_protocol,
            clientbound: HashMap::new(),
            serverbound: HashMap::new(),
            cancel_clientbound: HashSet::new(),
            cancel_serverbound: HashSet::new(),
        }
    }

    pub fn register(
        mut self,
        direction: Direction,
        packet_id: u16,
        handler: impl Fn(&mut PacketWrapper, &mut ConnState) -> Result<()> + Send + Sync + 'static,
    ) -> Translator {
        let name = self.name;
        let table = match direction {
            Direction::Clientbound => &mut self.clientbound,
            Direction::Serverbound => &mut self.serverbound,
        };
        assert!(
            table.insert(packet_id, Box::new(handler)).is_none(),
            "{}: duplicate {} handler for packet {}",
            name,
            direction.as_str(),
            packet_id
        );
        self
    }

    pub fn clientbound(
        self,
        packet_id: u16,
        handler: impl Fn(&mut PacketWrapper, &mut ConnState) -> Result<()> + Send + Sync + 'static,
    ) -> Translator {
        self.register(Direction::Clientbound, packet_id, handler)
    }

    pub fn serverbound(
        self,
        packet_id: u16,
        handler: impl Fn(&mut PacketWrapper, &mut ConnState) -> Result<()> + Send + Sync + 'static,
    ) -> Translator {
        self.register(Direction::Serverbound, packet_id, handler)
    }

    pub fn cancel(mut self, direction: Direction, packet_id: u16) -> Translator {
        match direction {
            Direction::Clientbound => self.cancel_clientbound.insert(packet_id),
            Direction::Serverbound => self.cancel_serverbound.insert(packet_id),
        };
        self
    }

    pub fn cancel_all(mut self, direction: Direction, packet_ids: &[u16]) -> Translator {
        for id in packet_ids {
            self = self.cancel(direction, *id);
        }
        self
    }

    pub fn is_cancelled(&self, direction: Direction, packet_id: u16) -> bool {
        match direction {
            Direction::Clientbound => self.cancel_clientbound.contains(&packet_id),
            Direction::Serverbound => self.cancel_serverbound.contains(&packet_id),
        }
    }

    pub fn handler(&self, direction: Direction, packet_id: u16) -> Option<&Handler> {
        match direction {
            Direction::Clientbound => self.clientbound.get(&packet_id),
            Direction::Serverbound => self.serverbound.get(&packet_id),
        }
    }

    pub fn touches(&self, direction: Direction, packet_id: u16) -> bool {
        self.is_cancelled(direction, packet_id) || self.handler(direction, packet_id).is_some()
    }

    pub fn handler_count(&self) -> usize {
        self.clientbound.len() + self.serverbound.len()
    }

    pub fn cancel_count(&self) -> usize {
        self.cancel_clientbound.len() + self.cancel_serverbound.len()
    }
}

impl std::fmt::Debug for Translator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Translator")
            .field("name", &self.name)
            .field("server_protocol", &self.server_protocol)
            .field("client_protocol", &self.client_protocol)
            .field("handlers", &self.handler_count())
            .field("cancels", &self.cancel_count())
            .finish()
    }
}
