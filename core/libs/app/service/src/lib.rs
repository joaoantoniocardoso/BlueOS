//! Async service kernel wiring sans-IO domains to comms adapters (D-04, D-10, D-12).
//!
//! Each service owns one tokio inbox task that holds [`App`] and applies [`Effect`]s. Adapters decode
//! Zenoh queries and samples into domain commands and never call [`App::handle`] directly.
//!
//! ## Minimal example
//!
//! ```no_run
//! use std::time::Duration;
//!
//! use blueos_comms::{Endpoint, Session};
//! use blueos_cqrs::{App, Decision, Domain, Effect, TimerId};
//! use blueos_idl::msg::blueos_msgs::{JobList, ServiceInfo, ServiceStatus};
//! use blueos_jobs::{JobGraph, Jobs};
//! use blueos_service::ServiceBuilder;
//!
//! struct ExampleDomain;
//!
//! #[derive(Clone, Default)]
//! struct ExampleSnapshot {
//!     counter: u32,
//! }
//!
//! #[derive(Clone)]
//! enum ExampleCommand {
//!     Increment,
//! }
//!
//! enum ExampleEvent {}
//! enum ExampleQuery {}
//! struct ExampleView;
//! #[derive(Clone)]
//! enum ExampleJobSpec {
//!     Work,
//! }
//! #[derive(Clone)]
//! enum ExampleIo {
//!     Delay(Duration),
//! }
//!
//! impl Domain for ExampleDomain {
//!     type Command = ExampleCommand;
//!     type Event = ExampleEvent;
//!     type Query = ExampleQuery;
//!     type View = ExampleView;
//!     type Snapshot = ExampleSnapshot;
//!     type IoRequest = ExampleIo;
//!     type JobSpec = ExampleJobSpec;
//!
//!     fn handle_command(
//!         snapshot: &mut Self::Snapshot,
//!         _jobs: &mut Jobs<Self::JobSpec>,
//!         command: Self::Command,
//!     ) -> Decision<Self> {
//!         match command {
//!             ExampleCommand::Increment => {
//!                 snapshot.counter += 1;
//!                 Decision::new()
//!             }
//!         }
//!     }
//!
//!     fn handle_query(
//!         _snapshot: &Self::Snapshot,
//!         _jobs: &Jobs<Self::JobSpec>,
//!         _query: Self::Query,
//!     ) -> Self::View {
//!         ExampleView
//!     }
//!
//!     fn io_from_job(_job_id: blueos_jobs::JobId, _job_spec: &Self::JobSpec) -> Self::IoRequest {
//!         ExampleIo::Delay(Duration::from_millis(1))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), blueos_service::ServiceError> {
//!     let session = Session::open("example", Endpoint::Local).await?;
//!     ServiceBuilder::<ExampleDomain>::new("example")
//!         .app(App::new(ExampleSnapshot::default()))
//!         .service_info(ServiceInfo {
//!             name: "example".into(),
//!             version: env!("CARGO_PKG_VERSION").into(),
//!             build: String::new(),
//!             capabilities: Vec::new(),
//!         })
//!         .status(|application| ServiceStatus {
//!             status: 2,
//!             detail: format!("counter={}", application.snapshot.counter),
//!         })
//!         .jobs(|application| JobList { jobs: Vec::new() })
//!         .command("Increment", |_| Ok(ExampleCommand::Increment))
//!         .io(|_application, request| async move {
//!             let ExampleIo::Delay(duration) = request;
//!             tokio::time::sleep(duration).await;
//!             Ok(ExampleCommand::Increment)
//!         })
//!         .session(session)
//!         .run()
//!         .await
//! }
//! ```

mod builder;
mod error;
mod runtime;

pub use builder::ServiceBuilder;
pub use error::ServiceError;
