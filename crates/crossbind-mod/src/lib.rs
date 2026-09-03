#![allow(missing_docs)]

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use bedrock_protocol::{
    describe_set_score_layout, diag, item_remap, translate, versions, ConnState, Direction as Way,
    Outcome, Registry,
};
use levilamina::prelude::*;

static REGISTRY: OnceLock<Registry> = OnceLock::new();

static CONNECTIONS: OnceLock<Mutex<HashMap<u64, ConnState>>> = OnceLock::new();

fn connections() -> &'static Mutex<HashMap<u64, ConnState>> {
    CONNECTIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

struct Crossbind;

impl LeviMod for Crossbind {
    fn on_load(_ctx: &ModContext) -> Result<Self> {
        Ok(Crossbind)
    }

    fn on_enable(&mut self, ctx: &ModContext) -> Result<()> {
        let logger = ctx.logger();
        let host = ctx.host();

        let server_protocol = host.protocol_version()?;
        logger.info(&format!(
            "server speaks {}",
            versions::describe(server_protocol)
        ));

        if !versions::is_translatable(server_protocol) {
            logger.warn(&format!(
                "{} is not a version crossbind can translate; staying inactive so \
                 clients still get a normal version-mismatch screen",
                versions::describe(server_protocol)
            ));
            logger.warn(&format!(
                "translatable server versions: {}",
                versions::TRANSLATABLE
                    .iter()
                    .map(|p| versions::describe(*p))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            return Ok(());
        }

        let registry = REGISTRY.get_or_init(|| bedrock_protocol::build_registry(server_protocol));
        if diag::enabled() {
            logger.info(&bedrock_protocol::describe_support(registry));
            logger.info(&describe_set_score_layout());
        }

        match item_remap::client_items() {
            Some(items) => logger.info(&format!(
                "item id translation ready ({} client items)",
                items.len()
            )),
            None => logger.warn(
                "the built-in client item table failed to parse; item ids will be \
                 forwarded unchanged and some items will be wrong",
            ),
        }

        ctx.packets()
            .intercept(Directions::Both, move |packet| {
                let way = match packet.direction() {
                    Direction::Outbound => Way::Clientbound,
                    Direction::Inbound => Way::Serverbound,
                };

                let mut guard = connections()
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let state = guard
                    .entry(packet.conn_id())
                    .or_insert_with(|| ConnState::new(server_protocol));

                let result = translate(
                    registry,
                    state,
                    way,
                    packet.packet_id() as u16,
                    packet.body(),
                );

                let notices = result.notices;
                drop(guard);

                for notice in &notices {
                    logger.info(notice);
                }

                match result.outcome {
                    Outcome::Unchanged => Verdict::Forward,
                    Outcome::Rewritten(bytes) => {
                        packet.set_body(&bytes);
                        Verdict::Forward
                    }
                    Outcome::Drop => Verdict::Drop,
                }
            })?
            .forget();

        ctx.packets()
            .on_connection(|conn_id, _address, state| {
                if state == ConnectionState::Closed {
                    if let Ok(mut guard) = connections().lock() {
                        guard.remove(&conn_id);
                    }
                }
            })?
            .forget();

        logger.info("crossbind enabled");
        Ok(())
    }

    fn on_disable(&mut self, ctx: &ModContext) -> Result<()> {
        if let Ok(mut guard) = connections().lock() {
            guard.clear();
        }
        ctx.logger().info("crossbind disabled");
        Ok(())
    }
}

levilamina::register_mod!(Crossbind);
