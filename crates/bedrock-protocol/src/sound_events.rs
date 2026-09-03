use std::collections::HashMap;
use std::sync::OnceLock;

const TABLE: &str = include_str!("../data/sound_events.tsv");

struct Index {
    by_id: HashMap<u32, &'static str>,
    by_name: HashMap<&'static str, u32>,
}

fn index() -> &'static Index {
    static CELL: OnceLock<Index> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut by_id = HashMap::new();
        let mut by_name = HashMap::new();
        for line in TABLE.lines() {
            let line = line.trim_end_matches('\r');
            if line.is_empty() {
                continue;
            }
            let Some((id, name)) = line.split_once('\t') else {
                continue;
            };
            let Ok(id) = id.parse::<u32>() else {
                continue;
            };
            by_id.insert(id, name);
            by_name.insert(name, id);
        }
        Index { by_id, by_name }
    })
}

pub fn id_to_name(id: u32) -> Option<&'static str> {
    index().by_id.get(&id).copied()
}

pub fn name_to_id(name: &str) -> Option<u32> {
    index().by_name.get(name).copied()
}

pub fn len() -> usize {
    index().by_id.len()
}

#[cfg(test)]
mod tests {
    #[test]
    fn table_parses_and_round_trips() {
        assert_eq!(super::len(), 563);
        assert_eq!(super::id_to_name(0), Some("item.use.on"));
        assert_eq!(super::name_to_id("item.use.on"), Some(0));
        for id in 0..=600u32 {
            if let Some(name) = super::id_to_name(id) {
                assert_eq!(
                    super::name_to_id(name),
                    Some(id),
                    "id {id} does not round trip"
                );
            }
        }
    }
}
