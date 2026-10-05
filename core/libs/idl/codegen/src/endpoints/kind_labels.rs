//! Wire and UI labels shared by every [`super::Kind`] variant.

use super::Kind;

struct KindProfile {
    zenoh_segment: &'static str,
    display_title: &'static str,
    wire_kind_name: &'static str,
    typescript_key_helper: &'static str,
    typescript_schema_parts: &'static [(&'static str, &'static str)],
}

const PROFILES: [KindProfile; 5] = [
    KindProfile {
        zenoh_segment: "command",
        display_title: "Job type",
        wire_kind_name: "job",
        typescript_key_helper: "commandKey",
        typescript_schema_parts: &[
            ("goalSchema", "Goal"),
            ("feedbackSchema", "Feedback"),
            ("resultSchema", "Result"),
        ],
    },
    KindProfile {
        zenoh_segment: "query",
        display_title: "Query",
        wire_kind_name: "query",
        typescript_key_helper: "queryKey",
        typescript_schema_parts: &[("requestSchema", "Request"), ("responseSchema", "Response")],
    },
    KindProfile {
        zenoh_segment: "query",
        display_title: "IO query",
        wire_kind_name: "query",
        typescript_key_helper: "queryKey",
        typescript_schema_parts: &[("requestSchema", "Request"), ("responseSchema", "Response")],
    },
    KindProfile {
        zenoh_segment: "state",
        display_title: "State",
        wire_kind_name: "state",
        typescript_key_helper: "stateKey",
        typescript_schema_parts: &[("messageSchema", "")],
    },
    KindProfile {
        zenoh_segment: "event",
        display_title: "Event",
        wire_kind_name: "event",
        typescript_key_helper: "eventKey",
        typescript_schema_parts: &[("messageSchema", "")],
    },
];

impl Kind {
    pub(crate) fn zenoh_segment(self) -> &'static str {
        PROFILES[self.index()].zenoh_segment
    }

    pub(crate) fn display_title(self) -> &'static str {
        PROFILES[self.index()].display_title
    }

    pub(crate) fn wire_kind_name(self) -> &'static str {
        PROFILES[self.index()].wire_kind_name
    }

    pub(crate) fn typescript_key_helper(self) -> &'static str {
        PROFILES[self.index()].typescript_key_helper
    }

    pub(crate) fn typescript_schema_parts(self) -> &'static [(&'static str, &'static str)] {
        PROFILES[self.index()].typescript_schema_parts
    }

    const STATE_PROFILE_INDEX: usize = 3;
    const EVENT_PROFILE_INDEX: usize = 4;

    fn index(self) -> usize {
        match self {
            Kind::Job => 0,
            Kind::Query => 1,
            Kind::IoQuery => 2,
            Kind::State => Self::STATE_PROFILE_INDEX,
            Kind::Event => Self::EVENT_PROFILE_INDEX,
        }
    }
}
