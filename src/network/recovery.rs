use bitter::{BitReader, LittleEndianReader};

use crate::bits::RlBits;
use crate::errors::{AttributeError, FrameError};
use crate::network::attributes::{AttributeDecoder, AttributeTag};
use crate::network::models::{
    ActorId, NetworkWarning, ObjectId, RecoveryKind, StreamId, UpdatedAttribute,
};
use crate::network::CacheInfo;

const MAX_SKIP_BITS: u32 = 64;

pub(crate) enum AttributeDecodeResult {
    Decoded(UpdatedAttribute),
    Recovered(NetworkWarning),
}

pub(crate) struct AttributeDecodeRequest<'a, 'b> {
    pub attr_decoder: &'a AttributeDecoder,
    pub bits: &'a mut LittleEndianReader<'b>,
    pub buf: &'a mut [u8],
    pub cache_info: &'a CacheInfo,
    pub objects: &'a [String],
    pub actor_id: ActorId,
    pub actor_object_id: ObjectId,
    pub stream_id: StreamId,
    pub frame_index: usize,
    pub recover_unknown_attributes: bool,
}

pub(crate) fn decode_attribute_update(
    req: AttributeDecodeRequest<'_, '_>,
) -> Result<AttributeDecodeResult, FrameError> {
    let attr = match req.cache_info.attributes.get(req.stream_id).copied() {
        Some(attr) => attr,
        None if req.recover_unknown_attributes => {
            if let Some(recovery) =
                recover_unknown_attribute(req.attr_decoder, req.bits, req.buf, req.cache_info, None)
            {
                return Ok(AttributeDecodeResult::Recovered(NetworkWarning {
                    frame: req.frame_index,
                    actor_id: req.actor_id,
                    actor_object_id: req.actor_object_id,
                    stream_id: req.stream_id,
                    attribute_object_id: None,
                    attribute_name: None,
                    recovery,
                }));
            }

            return Err(missing_attribute_error(&req));
        }
        None => return Err(missing_attribute_error(&req)),
    };

    if req.recover_unknown_attributes && attr.attribute == AttributeTag::NotImplemented {
        let attribute_name = req
            .objects
            .get(usize::from(attr.object_id))
            .map(String::as_str);

        if let Some(recovery) = recover_unknown_attribute(
            req.attr_decoder,
            req.bits,
            req.buf,
            req.cache_info,
            attribute_name,
        ) {
            return Ok(AttributeDecodeResult::Recovered(NetworkWarning {
                frame: req.frame_index,
                actor_id: req.actor_id,
                actor_object_id: req.actor_object_id,
                stream_id: req.stream_id,
                attribute_object_id: Some(attr.object_id),
                attribute_name: attribute_name.map(String::from),
                recovery,
            }));
        }

        return Err(missing_attribute_error(&req));
    }

    let attribute = req
        .attr_decoder
        .decode(attr.attribute, req.bits, req.buf)
        .map_err(|e| match e {
            AttributeError::Unimplemented => missing_attribute_error(&req),
            e => FrameError::AttributeError {
                actor: req.actor_id,
                actor_object: req.actor_object_id,
                attribute_stream: req.stream_id,
                error: e,
            },
        })?;

    Ok(AttributeDecodeResult::Decoded(UpdatedAttribute {
        actor_id: req.actor_id,
        stream_id: req.stream_id,
        object_id: attr.object_id,
        attribute,
    }))
}

fn missing_attribute_error(req: &AttributeDecodeRequest<'_, '_>) -> FrameError {
    FrameError::MissingAttribute {
        actor: req.actor_id,
        actor_object: req.actor_object_id,
        attribute_stream: req.stream_id,
    }
}

fn recover_unknown_attribute(
    attr_decoder: &AttributeDecoder,
    bits: &mut LittleEndianReader<'_>,
    buf: &mut [u8],
    cache_info: &CacheInfo,
    attribute_name: Option<&str>,
) -> Option<RecoveryKind> {
    for candidate in candidates(attribute_name) {
        let mut trial = *bits;
        if attr_decoder.decode(candidate.tag, &mut trial, buf).is_ok()
            && looks_aligned(trial, cache_info)
        {
            *bits = trial;
            return Some(RecoveryKind::GuessedAttribute {
                decoder: candidate.name,
            });
        }
    }

    // Last resort: consume a small bounded number of bits and keep the first
    // locally plausible alignment. This is intentionally narrow because there
    // is no generic length prefix for network attributes.
    for skipped in 1..=MAX_SKIP_BITS {
        let mut trial = *bits;
        trial.read_bits(skipped)?;
        if looks_aligned(trial, cache_info) {
            *bits = trial;
            return Some(RecoveryKind::SkippedBits { bits: skipped });
        }
    }

    None
}

fn looks_aligned(mut bits: LittleEndianReader<'_>, cache_info: &CacheInfo) -> bool {
    let Some(has_next_attr) = bits.read_bit() else {
        return false;
    };

    if !has_next_attr {
        return true;
    }

    bits.refill_lookahead();
    if bits.lookahead_bits() < cache_info.prop_id_bits + 1 {
        return false;
    }

    let stream_id = StreamId(
        bits.peek_bits_max_computed(cache_info.prop_id_bits, u64::from(cache_info.max_prop_id))
            as i32,
    );
    cache_info.attributes.get(stream_id).is_some()
}

#[derive(Clone, Copy)]
struct Candidate {
    name: &'static str,
    tag: AttributeTag,
}

fn candidates(attribute_name: Option<&str>) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(10);
    if let Some(name) = attribute_name {
        add_name_candidates(name, &mut candidates);
    }

    for candidate in [
        Candidate {
            name: "boolean",
            tag: AttributeTag::Boolean,
        },
        Candidate {
            name: "byte",
            tag: AttributeTag::Byte,
        },
        Candidate {
            name: "int",
            tag: AttributeTag::Int,
        },
        Candidate {
            name: "float",
            tag: AttributeTag::Float,
        },
        Candidate {
            name: "active_actor",
            tag: AttributeTag::ActiveActor,
        },
        Candidate {
            name: "rigid_body",
            tag: AttributeTag::RigidBody,
        },
        Candidate {
            name: "string",
            tag: AttributeTag::String,
        },
    ] {
        push_unique(&mut candidates, candidate);
    }

    candidates
}

fn add_name_candidates(name: &str, candidates: &mut Vec<Candidate>) {
    let property = name
        .rsplit_once(':')
        .map(|(_, property)| property)
        .unwrap_or(name)
        .to_ascii_lowercase();

    if property.starts_with('b') {
        push_unique(
            candidates,
            Candidate {
                name: "boolean",
                tag: AttributeTag::Boolean,
            },
        );
    }

    if property.contains("name") || property.contains("title") || property.contains("text") {
        push_unique(
            candidates,
            Candidate {
                name: "string",
                tag: AttributeTag::String,
            },
        );
    }

    if property.contains("rbstate")
        || property.contains("rigid")
        || property.contains("location")
        || property.contains("velocity")
    {
        push_unique(
            candidates,
            Candidate {
                name: "rigid_body",
                tag: AttributeTag::RigidBody,
            },
        );
    }

    if property.contains("actor") || property.contains("owner") || property.contains("pri") {
        push_unique(
            candidates,
            Candidate {
                name: "active_actor",
                tag: AttributeTag::ActiveActor,
            },
        );
    }

    if property.contains("ping")
        || property.contains("team")
        || property.contains("platform")
        || property.contains("byte")
    {
        push_unique(
            candidates,
            Candidate {
                name: "byte",
                tag: AttributeTag::Byte,
            },
        );
    }

    if property.contains("score")
        || property.contains("count")
        || property.contains("num")
        || property.contains("index")
        || property.ends_with("id")
    {
        push_unique(
            candidates,
            Candidate {
                name: "int",
                tag: AttributeTag::Int,
            },
        );
    }
}

fn push_unique(candidates: &mut Vec<Candidate>, candidate: Candidate) {
    if !candidates.iter().any(|x| x.tag == candidate.tag) {
        candidates.push(candidate);
    }
}
