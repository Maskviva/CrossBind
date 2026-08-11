#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub protocol: u32,
    pub name: &'static str,
}

pub const VERSIONS: &[Version] = &[
    Version { protocol: 2168, name: "1.26.40" },
    Version { protocol: 1001, name: "1.26.30" },
    Version { protocol: 975, name: "1.26.20" },
    Version { protocol: 944, name: "1.26.10" },
    Version { protocol: 924, name: "1.26.0" },
    Version { protocol: 898, name: "1.21.130" },
    Version { protocol: 860, name: "1.21.124" },
    Version { protocol: 859, name: "1.21.120" },
    Version { protocol: 844, name: "1.21.111" },
    Version { protocol: 827, name: "1.21.100" },
    Version { protocol: 819, name: "1.21.93" },
    Version { protocol: 818, name: "1.21.90" },
    Version { protocol: 800, name: "1.21.80" },
    Version { protocol: 786, name: "1.21.70" },
    Version { protocol: 776, name: "1.21.60" },
    Version { protocol: 766, name: "1.21.50" },
    Version { protocol: 748, name: "1.21.40" },
    Version { protocol: 729, name: "1.21.30" },
];

pub const TRANSLATABLE: &[u32] = &[859, 860, 898, 924, 944, 975, 1001, 2168];

pub fn name_of(protocol: u32) -> Option<&'static str> {
    VERSIONS
        .iter()
        .find(|v| v.protocol == protocol)
        .map(|v| v.name)
}

pub fn describe(protocol: u32) -> String {
    match name_of(protocol) {
        Some(name) => format!("{name} (protocol {protocol})"),
        None => format!("protocol {protocol}"),
    }
}

pub fn is_translatable(protocol: u32) -> bool {
    TRANSLATABLE.contains(&protocol)
}

pub fn nearest_translatable_below(protocol: u32) -> Option<u32> {
    TRANSLATABLE
        .iter()
        .copied()
        .filter(|candidate| *candidate <= protocol)
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_translatable_protocol_has_a_name() {
        for protocol in TRANSLATABLE {
            assert!(
                name_of(*protocol).is_some(),
                "protocol {protocol} is translatable but unnamed"
            );
        }
    }

    #[test]
    fn version_table_is_sorted_and_unique() {
        for pair in VERSIONS.windows(2) {
            assert!(
                pair[0].protocol > pair[1].protocol,
                "VERSIONS must be strictly descending: {} then {}",
                pair[0].protocol,
                pair[1].protocol
            );
        }
    }

    #[test]
    fn names_below_924_match_gophertunnel_history_not_1_25_x() {
        for (protocol, name) in [
            (898, "1.21.130"),
            (860, "1.21.124"),
            (859, "1.21.120"),
            (844, "1.21.111"),
            (827, "1.21.100"),
            (819, "1.21.93"),
            (818, "1.21.90"),
            (800, "1.21.80"),
            (786, "1.21.70"),
            (776, "1.21.60"),
            (766, "1.21.50"),
            (748, "1.21.40"),
            (729, "1.21.30"),
        ] {
            assert_eq!(name_of(protocol), Some(name), "protocol {protocol}");
            assert!(
                !name_of(protocol).unwrap().starts_with("1.25."),
                "gophertunnel's history has no 1.25.x release; the real line \
                 runs 1.21.130 -> 1.26.0 with nothing between them",
            );
        }
    }

    #[test]
    fn nearest_below_picks_the_right_neighbour() {
        assert_eq!(nearest_translatable_below(2200), Some(2168));
        assert_eq!(nearest_translatable_below(1000), Some(975));
        assert_eq!(nearest_translatable_below(900), Some(898));
        assert_eq!(nearest_translatable_below(859), Some(859));
        assert_eq!(nearest_translatable_below(800), None);
    }
}
