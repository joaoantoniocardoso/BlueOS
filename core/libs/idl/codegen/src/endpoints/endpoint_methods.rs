//! Endpoint naming and generated function lists (keeps `Endpoint` data-only).

use super::{Endpoint, Kind};

impl Endpoint {
    pub(crate) fn key(&self, service: &str) -> String {
        format!(
            "blueos/v1/{service}/{}/{}",
            self.kind.zenoh_segment(),
            self.name
        )
    }

    pub(crate) fn title(&self) -> String {
        format!("{} `{}`", self.kind.display_title(), self.name)
    }

    pub(crate) fn is_handled(&self) -> bool {
        self.custom || self.kind == Kind::IoQuery
    }

    /// The Job id parameter of its Goal mapping: a Job type that declares its nature hands it to the Domain.
    pub(crate) fn job_id_parameter(&self) -> &'static str {
        if self.nature.is_some() {
            "job_id: JobId, "
        } else {
            ""
        }
    }

    /// The functions it generates in `Conversions` and `Handlers`, which must be unique in a Service.
    pub(crate) fn functions(&self) -> Vec<String> {
        let function = &self.function;
        match self.kind {
            Kind::Job => vec![
                function.clone(),
                format!("{function}_feedback"),
                format!("{function}_result"),
            ],
            Kind::Query => vec![function.clone(), format!("{function}_response")],
            Kind::IoQuery | Kind::State | Kind::Event => vec![function.clone()],
        }
    }

    /// Its kind in `info`: an IO query is a Query, because clients cannot tell them apart.
    pub(crate) fn kind_name(&self) -> &'static str {
        self.kind.wire_kind_name()
    }
}
