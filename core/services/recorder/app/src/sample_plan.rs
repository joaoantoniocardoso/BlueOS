//! D-24 schema resolution for backbone samples (app layer; adapters stay IO-only).

use std::{borrow::Cow, collections::BTreeMap, sync::Arc};

use bytes::Bytes;

use blueos_comms::Payload;
use blueos_recorder_mcap::{
    ChannelDescriptor, ChannelRoute, cached_descriptor, channel_descriptor_for_ros2_type,
    channel_descriptor_for_sample,
};
use blueos_recorder_schema_gate::{Ros2ddsGate, is_ros2_schema_candidate};
use blueos_ros2_names::parse_rmw_zenoh_data_key;

/// How the data plane should handle one backbone sample.
pub(crate) enum SampleWritePlan {
    /// Write immediately with this route and descriptor.
    Ready {
        /// Writer route.
        route: ChannelRoute,
        /// Channel metadata (cached by route).
        descriptor: Arc<ChannelDescriptor>,
    },
    /// Sample may arrive before its ros2dds liveliness token.
    NeedsRos2Gate,
    /// Not a recordable payload for this encoding/topic (see module docs).
    Skip,
}

/// Resolves schema in order: encoding suffix, rmw_zenoh key, gate-known ros2dds type, else gate or skip.
///
/// `SampleWritePlan::Skip` drops samples that are not BlueOS/recorder-internal exclusions but also not
/// MCAP-recordable: wrong wire shape (no CDR header on a zenoh/bytes ros2 candidate), non-object JSON,
/// unknown encodings, and non-ros2 octet-stream keys that do not match earlier lanes.
pub(crate) fn plan_sample_write<GatePayload>(
    topic: &str,
    encoding: &str,
    payload: &Payload,
    gate: &Ros2ddsGate<GatePayload>,
    cache: &mut BTreeMap<ChannelRoute, Arc<ChannelDescriptor>>,
) -> SampleWritePlan {
    let default_route = ChannelRoute::for_topic(topic);
    if may_resolve_from_encoding_suffix(encoding) {
        if let Some(descriptor) = cache.get(&default_route) {
            return SampleWritePlan::Ready {
                route: default_route,
                descriptor: Arc::clone(descriptor),
            };
        }
        if let Some(descriptor) = channel_descriptor_for_sample(topic, encoding, payload) {
            let descriptor = cached_descriptor(cache, &default_route, || descriptor);
            return SampleWritePlan::Ready {
                route: default_route,
                descriptor,
            };
        }
    }

    if let Some(parsed) = parse_rmw_zenoh_data_key(topic) {
        let type_name = parsed.type_name;
        let route = ChannelRoute::typed(topic, &type_name);
        if let Some(descriptor) = cache.get(&route) {
            return SampleWritePlan::Ready {
                route,
                descriptor: Arc::clone(descriptor),
            };
        }
        let descriptor = cached_descriptor(cache, &route, || {
            channel_descriptor_for_ros2_type(topic, &type_name)
        });
        return SampleWritePlan::Ready { route, descriptor };
    }

    if let Some(type_name) = gate.type_name(topic) {
        let route = ChannelRoute::typed(topic, type_name);
        if let Some(descriptor) = cache.get(&route) {
            return SampleWritePlan::Ready {
                route,
                descriptor: Arc::clone(descriptor),
            };
        }
        let descriptor = cached_descriptor(cache, &route, || {
            channel_descriptor_for_ros2_type(topic, type_name)
        });
        return SampleWritePlan::Ready { route, descriptor };
    }

    if gate.tracks_topic(topic) {
        return SampleWritePlan::NeedsRos2Gate;
    }

    let prefix = cdr_header_prefix(payload);
    if is_ros2_schema_candidate(topic, encoding, prefix.as_ref()) {
        return SampleWritePlan::NeedsRos2Gate;
    }
    SampleWritePlan::Skip
}

fn may_resolve_from_encoding_suffix(encoding: &str) -> bool {
    encoding.starts_with("application/cdr;")
        || encoding.starts_with("application/json")
        || encoding.starts_with("application/octet-stream")
}

fn cdr_header_prefix(payload: &Payload) -> Cow<'_, [u8]> {
    if let Some(bytes) = payload.downcast_ref::<Bytes>() {
        let end = bytes.len().min(4);
        return Cow::Borrowed(&bytes[..end]);
    }
    let bytes = payload.to_bytes();
    let end = bytes.len().min(4);
    match bytes {
        Cow::Borrowed(slice) => Cow::Borrowed(&slice[..end]),
        Cow::Owned(vec) => Cow::Owned(vec[..end].to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CDR_HEADER: [u8; 4] = [0x00, 0x01, 0x00, 0x00];

    #[test]
    fn cache_hit_avoids_descriptor_build() {
        let mut cache = BTreeMap::new();
        let route = ChannelRoute::for_topic("video/camera1/stream");
        let descriptor = Arc::new(
            channel_descriptor_for_sample(
                "video/camera1/stream",
                "application/octet-stream",
                &Payload::new(Bytes::from_static(b"x")),
            )
            .expect("descriptor"),
        );
        cache.insert(route.clone(), Arc::clone(&descriptor));
        let gate = Ros2ddsGate::<Payload>::default();
        let plan = plan_sample_write(
            "video/camera1/stream",
            "application/octet-stream",
            &Payload::new(Bytes::from_static(b"y")),
            &gate,
            &mut cache,
        );
        match plan {
            SampleWritePlan::Ready {
                route: planned_route,
                descriptor: planned,
            } => {
                assert_eq!(planned_route, route);
                assert!(Arc::ptr_eq(&planned, &descriptor));
            }
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn zenoh_bytes_without_cdr_header_skips() {
        let mut cache = BTreeMap::new();
        let gate = Ros2ddsGate::<Payload>::default();
        let plan = plan_sample_write(
            "chatter",
            "zenoh/bytes",
            &Payload::new(Bytes::from_static(b"no-header")),
            &gate,
            &mut cache,
        );
        assert!(matches!(plan, SampleWritePlan::Skip));
    }

    #[test]
    fn zenoh_bytes_with_cdr_header_uses_gate() {
        let mut cache = BTreeMap::new();
        let gate = Ros2ddsGate::<Payload>::default();
        let plan = plan_sample_write(
            "chatter",
            "zenoh/bytes",
            &Payload::new(Bytes::from_static(&CDR_HEADER)),
            &gate,
            &mut cache,
        );
        assert!(matches!(plan, SampleWritePlan::NeedsRos2Gate));
    }
}
