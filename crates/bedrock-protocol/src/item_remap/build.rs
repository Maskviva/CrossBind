use super::table::ClientItems;
use core::fmt;
use std::collections::{HashMap, HashSet};

pub struct ItemRemap {
    to_client: HashMap<i32, i32>,
    to_server: HashMap<i32, i32>,
    report: BuildReport,
}

impl ItemRemap {
    pub fn build(client: &ClientItems, server: &[(String, i32)]) -> ItemRemap {
        let mut report = BuildReport::default();
        let mut to_client: HashMap<i32, i32> = HashMap::with_capacity(server.len());
        let mut to_server: HashMap<i32, i32> = HashMap::with_capacity(server.len());

        let mut claimed: HashSet<i32> = HashSet::with_capacity(server.len());
        let mut leftovers: Vec<i32> = Vec::new();
        for (name, server_id) in server {
            if *server_id == AIR {
                continue;
            }
            match client.by_name.get(name.as_str()) {
                Some(client_id) => {
                    if *client_id == *server_id {
                        report.agreed += 1;
                    } else {
                        report.renumbered += 1;
                    }
                    claimed.insert(*client_id);
                    to_client.insert(*server_id, *client_id);
                    to_server.insert(*client_id, *server_id);
                }
                None => leftovers.push(*server_id),
            }
        }

        let mut next_positive = client
            .used
            .iter()
            .copied()
            .filter(|i| *i > 0)
            .max()
            .unwrap_or(0)
            + 1;
        let mut next_negative = client
            .used
            .iter()
            .copied()
            .filter(|i| *i < 0)
            .min()
            .unwrap_or(0)
            - 1;

        for server_id in leftovers {
            if !claimed.contains(&server_id) && !client.used.contains(&server_id) {
                report.kept += 1;
                claimed.insert(server_id);
                to_client.insert(server_id, server_id);
                to_server.insert(server_id, server_id);
                continue;
            }

            let fresh = if server_id < 0 {
                allocate(&mut next_negative, &claimed, &client.used, -1)
            } else {
                allocate(&mut next_positive, &claimed, &client.used, 1)
            };
            match fresh {
                Some(id) => {
                    report.relocated += 1;
                    claimed.insert(id);
                    to_client.insert(server_id, id);
                    to_server.insert(id, server_id);
                }
                None => {
                    report.unplaceable += 1;
                }
            }
        }

        ItemRemap {
            to_client,
            to_server,
            report,
        }
    }

    pub fn to_client(&self, id: i32) -> i32 {
        if id == AIR {
            return AIR;
        }
        self.to_client.get(&id).copied().unwrap_or(id)
    }

    pub fn to_server(&self, id: i32) -> i32 {
        if id == AIR {
            return AIR;
        }
        self.to_server.get(&id).copied().unwrap_or(id)
    }

    pub fn report(&self) -> BuildReport {
        self.report
    }

    pub fn len(&self) -> usize {
        self.to_client.len()
    }

    pub fn is_empty(&self) -> bool {
        self.to_client.is_empty()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BuildReport {
    pub renumbered: usize,
    pub agreed: usize,
    pub kept: usize,
    pub relocated: usize,
    pub unplaceable: usize,
}

impl fmt::Display for BuildReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} renumbered, {} already agreed, {} server-only kept, {} relocated",
            self.renumbered, self.agreed, self.kept, self.relocated
        )?;
        if self.unplaceable != 0 {
            write!(f, ", {} could not be placed", self.unplaceable)?;
        }
        Ok(())
    }
}

fn allocate(
    cursor: &mut i32,
    claimed: &HashSet<i32>,
    used: &HashSet<i32>,
    step: i32,
) -> Option<i32> {
    while (I16_MIN..=I16_MAX).contains(cursor) {
        let candidate = *cursor;
        *cursor += step;
        if candidate != AIR && !claimed.contains(&candidate) && !used.contains(&candidate) {
            return Some(candidate);
        }
    }
    None
}

pub(crate) const AIR: i32 = 0;

pub(crate) const I16_MIN: i32 = i16::MIN as i32;

pub(crate) const I16_MAX: i32 = i16::MAX as i32;
