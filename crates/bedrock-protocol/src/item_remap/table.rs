use super::build::{I16_MAX, I16_MIN};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct ClientItems {
    pub(super) by_name: HashMap<String, i32>,
    pub(super) used: HashSet<i32>,
}

impl ClientItems {
    pub fn from_registry_tsv(text: &str) -> std::result::Result<ClientItems, RemapLoadError> {
        let mut lines = text.lines().enumerate();
        let header = loop {
            match lines.next() {
                None => return Err(RemapLoadError::MissingHeader),
                Some((_, line)) if line.trim().is_empty() => continue,
                Some((_, line)) => break line,
            }
        };
        let columns: Vec<&str> = header.trim().split('\t').map(str::trim).collect();
        let (name_at, id_at) = match (
            columns.iter().position(|c| *c == "name"),
            columns.iter().position(|c| *c == "id"),
        ) {
            (Some(n), Some(i)) => (n, i),
            _ => return Err(RemapLoadError::BadHeader(header.trim().to_owned())),
        };

        let mut by_name: HashMap<String, i32> = HashMap::new();
        let mut used: HashSet<i32> = HashSet::new();
        let mut owner: HashMap<i32, String> = HashMap::new();

        for (index, line) in lines {
            let number = index + 1;
            if line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            let name = fields.get(name_at).map(|s| s.trim()).unwrap_or("");
            let raw_id = match fields.get(id_at) {
                Some(v) => v.trim(),
                None => {
                    return Err(RemapLoadError::BadRow {
                        line: number,
                        reason: "row is missing the id column".to_owned(),
                    })
                }
            };
            if name.is_empty() {
                return Err(RemapLoadError::BadRow {
                    line: number,
                    reason: "empty item name".to_owned(),
                });
            }
            let id: i32 = raw_id.parse().map_err(|_| RemapLoadError::BadRow {
                line: number,
                reason: format!("id `{raw_id}` is not an integer"),
            })?;
            if !(I16_MIN..=I16_MAX).contains(&id) {
                return Err(RemapLoadError::BadRow {
                    line: number,
                    reason: format!("id {id} does not fit the i16 the registry uses"),
                });
            }
            if by_name.insert(name.to_owned(), id).is_some() {
                return Err(RemapLoadError::DuplicateName {
                    name: name.to_owned(),
                });
            }
            if let Some(first) = owner.insert(id, name.to_owned()) {
                return Err(RemapLoadError::DuplicateId {
                    id,
                    first,
                    second: name.to_owned(),
                });
            }
            used.insert(id);
        }

        if by_name.is_empty() {
            return Err(RemapLoadError::Empty);
        }
        Ok(ClientItems { by_name, used })
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

const CLIENT_TABLE_V2168: &str = include_str!("../../data/item_registry_v2168.tsv");

static CLIENT_ITEMS: OnceLock<Option<ClientItems>> = OnceLock::new();

pub fn client_items() -> Option<&'static ClientItems> {
    CLIENT_ITEMS
        .get_or_init(|| ClientItems::from_registry_tsv(CLIENT_TABLE_V2168).ok())
        .as_ref()
}

#[derive(Debug)]
pub enum RemapLoadError {
    MissingHeader,
    BadHeader(String),
    BadRow {
        line: usize,
        reason: String,
    },
    DuplicateName {
        name: String,
    },
    DuplicateId {
        id: i32,
        first: String,
        second: String,
    },
    Empty,
}

impl fmt::Display for RemapLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemapLoadError::MissingHeader => f.write_str("client item table is empty"),
            RemapLoadError::BadHeader(got) => write!(
                f,
                "client item table needs a `name` and an `id` column, got `{got}`"
            ),
            RemapLoadError::BadRow { line, reason } => {
                write!(f, "client item table line {line}: {reason}")
            }
            RemapLoadError::DuplicateName { name } => {
                write!(f, "client item table lists {name} twice")
            }
            RemapLoadError::DuplicateId { id, first, second } => write!(
                f,
                "client item table gives id {id} to both {first} and {second}"
            ),
            RemapLoadError::Empty => f.write_str("client item table has a header but no rows"),
        }
    }
}
