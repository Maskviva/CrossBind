use std::collections::HashMap;

use crate::versions;

#[derive(Debug, Clone)]
pub struct ConnState {
    pub client_protocol: u32,
    pub server_protocol: u32,
    pub warned_unsupported: bool,
    pub notices: Vec<String>,
    pub item_ids: HashMap<String, i32>,
    pub item_names: HashMap<i32, String>,
}

impl ConnState {
    pub fn new(server_protocol: u32) -> ConnState {
        ConnState {
            client_protocol: 0,
            server_protocol,
            warned_unsupported: false,
            notices: Vec::new(),
            item_ids: HashMap::new(),
            item_names: HashMap::new(),
        }
    }

    pub fn is_identified(&self) -> bool {
        self.client_protocol != 0
    }

    pub fn is_native(&self) -> bool {
        self.client_protocol == self.server_protocol
    }

    pub fn is_supported(&self) -> bool {
        self.is_native()
            || (versions::is_translatable(self.client_protocol)
                && versions::is_translatable(self.server_protocol))
    }

    pub fn describe_client(&self) -> String {
        if self.is_identified() {
            versions::describe(self.client_protocol)
        } else {
            "unidentified client".to_owned()
        }
    }
}
