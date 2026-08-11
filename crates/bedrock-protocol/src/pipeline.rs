use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

use crate::connection::ConnState;
use crate::direction::Direction;
use crate::manager::Registry;
use crate::packet_ids;
use crate::translator::Translator;
use bedrock_codec::prelude::{Bool, Str};
use bedrock_codec::{Codec, PacketWrapper, Reader};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Unchanged,
    Rewritten(Vec<u8>),
    Drop,
}

pub struct Translation {
    pub outcome: Outcome,
    pub notices: Vec<String>,
}

impl Translation {
    fn plain(outcome: Outcome) -> Translation {
        Translation {
            outcome,
            notices: Vec::new(),
        }
    }
}

fn trace_limit() -> u32 {
    static LIMIT: OnceLock<u32> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        let on = std::env::var("CROSSBIND_TRACE")
            .map(|v| {
                let v = v.trim().to_ascii_lowercase();
                v == "1" || v == "true" || v == "on" || v == "yes"
            })
            .unwrap_or(false);
        if !on {
            return 0;
        }
        std::env::var("CROSSBIND_TRACE_LIMIT")
            .ok()
            .and_then(|v| v.trim().parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(200_000)
    })
}

static TRACE_SEQ: AtomicU32 = AtomicU32::new(0);

fn hex_head(bytes: &[u8], n: usize) -> String {
    bytes
        .iter()
        .take(n)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn translate(
    registry: &Registry,
    state: &mut ConnState,
    direction: Direction,
    packet_id: u16,
    body: &[u8],
) -> Translation {
    let mut result = translate_inner(registry, state, direction, packet_id, body);

    let limit = trace_limit();
    if limit != 0 {
        let seq = TRACE_SEQ.fetch_add(1, Ordering::Relaxed);
        if seq == 0 {
            result.notices.push(format!(
                "packet trace on (CROSSBIND_TRACE=1), cap {limit} lines; unset it or set CROSSBIND_TRACE=0 to turn off"
            ));
            result.notices.push(format!(
                "SubChunk mode {} (override with CROSSBIND_SUBCHUNK)",
                crate::steps::v1001_v2168::sub_chunk_mode_label()
            ));
            result.notices.push(format!(
                "blob cache {} (override with CROSSBIND_BLOB_CACHE)",
                if crate::steps::v975_v1001::blob_cache_enabled() {
                    "on"
                } else {
                    "off — sub-chunks sent inline"
                }
            ));
        }
        if seq < limit {
            let what = match &result.outcome {
                Outcome::Unchanged => format!("forward {} B", body.len()),
                Outcome::Rewritten(bytes) => match packet_id {
                    58 => format!(
                        "rewrite {} B -> {} B head={}",
                        body.len(),
                        bytes.len(),
                        hex_head(bytes, 16)
                    ),
                    174 => format!(
                        "rewrite {} B -> {} B in={} out={}",
                        body.len(),
                        bytes.len(),
                        hex_head(body, 32),
                        hex_head(bytes, 32)
                    ),
                    135 => format!(
                        "rewrite {} B -> {} B in={} out={}",
                        body.len(),
                        bytes.len(),
                        hex_head(body, 12),
                        hex_head(bytes, 12)
                    ),
                    _ => format!("rewrite {} B -> {} B", body.len(), bytes.len()),
                },
                Outcome::Drop => "DROP".to_owned(),
            };
            result.notices.push(format!(
                "trace {seq} {} id={packet_id} {} [{what}]",
                direction.as_str(),
                packet_ids::label(packet_id),
            ));
        }

        if packet_id == packet_ids::ids::DISCONNECT {
            let parsed = (|| -> bedrock_codec::Result<(bool, String)> {
                let mut r = Reader::new(body);
                let hide = Bool::read(&mut r)?;
                let msg = Str::read(&mut r)?;
                Ok((hide, msg))
            })();
            let line = match parsed {
                Ok((hide, msg)) => format!(
                    "trace {} Disconnect ({} B, hide_screen={}): {msg}",
                    direction.as_str(),
                    body.len(),
                    hide,
                ),
                Err(_) => {
                    let raw: String = body
                        .iter()
                        .take(240)
                        .map(|b| {
                            if b.is_ascii_graphic() || *b == b' ' {
                                *b as char
                            } else {
                                '.'
                            }
                        })
                        .collect();
                    format!(
                        "trace {} Disconnect body ({} B, unparseable): {raw}",
                        direction.as_str(),
                        body.len(),
                    )
                }
            };
            result.notices.push(line);
        }
    }

    result
}

fn translate_inner(
    registry: &Registry,
    state: &mut ConnState,
    direction: Direction,
    packet_id: u16,
    body: &[u8],
) -> Translation {
    let mut current: Option<Vec<u8>> = None;
    let mut notices: Vec<String> = Vec::new();

    match run_steps(
        registry.base(),
        state,
        direction,
        packet_id,
        body,
        &mut current,
        &mut notices,
    ) {
        StepFlow::Continue => {}
        StepFlow::Drop => {
            return Translation {
                outcome: Outcome::Drop,
                notices,
            }
        }
        StepFlow::Failed => {
            return Translation {
                outcome: Outcome::Unchanged,
                notices,
            }
        }
    }

    if state.is_identified() && !state.is_native() {
        match registry.chain(state.client_protocol) {
            Some(chain) => {
                let steps = match direction {
                    Direction::Serverbound => &chain.serverbound,
                    Direction::Clientbound => &chain.clientbound,
                };
                match run_steps(
                    steps,
                    state,
                    direction,
                    packet_id,
                    body,
                    &mut current,
                    &mut notices,
                ) {
                    StepFlow::Continue => {}
                    StepFlow::Drop => {
                        return Translation {
                            outcome: Outcome::Drop,
                            notices,
                        }
                    }
                    StepFlow::Failed => {
                        return Translation {
                            outcome: Outcome::Unchanged,
                            notices,
                        }
                    }
                }
            }
            None => {
                if !state.warned_unsupported {
                    state.warned_unsupported = true;
                    notices.push(format!(
                        "no translation path from {} to {}; forwarding untranslated",
                        crate::versions::describe(state.client_protocol),
                        crate::versions::describe(state.server_protocol),
                    ));
                }
            }
        }
    }

    if !state.notices.is_empty() {
        notices.append(&mut state.notices);
    }

    match current {
        Some(bytes) => Translation {
            outcome: Outcome::Rewritten(bytes),
            notices,
        },
        None if notices.is_empty() => Translation::plain(Outcome::Unchanged),
        None => Translation {
            outcome: Outcome::Unchanged,
            notices,
        },
    }
}

enum StepFlow {
    Continue,
    Drop,
    Failed,
}

fn run_steps(
    steps: &[std::sync::Arc<Translator>],
    state: &mut ConnState,
    direction: Direction,
    packet_id: u16,
    original: &[u8],
    current: &mut Option<Vec<u8>>,
    notices: &mut Vec<String>,
) -> StepFlow {
    for step in steps {
        if step.is_cancelled(direction, packet_id) {
            return StepFlow::Drop;
        }
        let Some(handler) = step.handler(direction, packet_id) else {
            continue;
        };

        let produced = {
            let input: &[u8] = current.as_deref().unwrap_or(original);
            let mut wrapper = PacketWrapper::new(input);
            match handler(&mut wrapper, state) {
                Ok(()) => {
                    if wrapper.is_cancelled() {
                        return StepFlow::Drop;
                    }
                    wrapper.finish()
                }
                Err(err) => {
                    notices.push(format!(
                        "{}: failed to translate {} {}: {err}",
                        step.name,
                        direction.as_str(),
                        packet_ids::label(packet_id),
                    ));
                    return StepFlow::Failed;
                }
            }
        };
        *current = Some(produced);
    }
    StepFlow::Continue
}
