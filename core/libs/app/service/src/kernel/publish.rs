//! State and Event publication.

use bytes::Bytes;

use blueos_api::{
    Message, cdr_encoding, event_key, job_feedback_key, job_history_key, job_result_key, jobs_key,
    state_key,
};
use blueos_comms::Sample;
use blueos_domain::Domain;
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{JobFeedbackList, JobList, JobResult, ServiceMetrics},
};

use crate::{builder::job_list, sync::lock_unpoisoned};

use super::{
    endpoints::warn_on_failure,
    types::{EncodedJobOutput, Kernel, METRICS, SendError},
};

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    pub(crate) async fn publish_settings(&self) {
        let Some(settings) = &self.settings else {
            return;
        };
        let sent: Result<(), SendError> = async {
            let payload =
                Bytes::from(lock_unpoisoned(&settings.driver).encode_state(&self.snapshot)?);
            if settings.latest.borrow().as_ref() == Some(&payload) {
                return Ok(());
            }
            let sample = Sample::new(
                settings.key.as_str(),
                Bytes::clone(&payload),
                &settings.encoding,
            );
            self.backend.publish(sample).await?;
            settings.latest.send_replace(Some(payload));
            Ok(())
        }
        .await;
        warn_on_failure("State", &settings.key, sent);
    }

    pub(crate) async fn publish_metrics(&self) {
        self.runtime_gauges.sample();
        let key = state_key(self.service, METRICS);
        let sent: Result<(), SendError> = async {
            let payload = Bytes::from(self.metrics.service_metrics().encode()?);
            if self.metrics_latest.borrow().as_ref() == Some(&payload) {
                return Ok(());
            }
            let encoding = cdr_encoding(ServiceMetrics::SCHEMA_NAME);
            let sample = Sample::new(key.as_str(), Bytes::clone(&payload), &encoding);
            self.backend.publish(sample).await?;
            self.metrics_latest.send_replace(Some(payload));
            Ok(())
        }
        .await;
        warn_on_failure("State", &key, sent);
    }

    pub(crate) async fn publish_jobs(&self) {
        let key = jobs_key(self.service);
        let sent: Result<(), SendError> = async {
            let payload = Bytes::from(job_list(self.jobs()).encode()?);
            if self.jobs_latest.borrow().as_ref() == Some(&payload) {
                return Ok(());
            }
            let encoding = cdr_encoding(JobList::SCHEMA_NAME);
            let sample = Sample::new(key.as_str(), Bytes::clone(&payload), &encoding);
            self.backend.publish(sample).await?;
            self.jobs_latest.send_replace(Some(payload));
            Ok(())
        }
        .await;
        warn_on_failure("State", &key, sent);
    }

    /// Publishes each Job type's Feedback State when it changed and keeps its history for the Query. Returns each
    /// Job type's Job result Events, for [`Self::publish_job_results`] once the Command is acknowledged.
    pub(crate) async fn publish_job_feedback(
        &self,
        outputs: Vec<EncodedJobOutput>,
    ) -> Vec<Vec<Result<Vec<u8>, IdlError>>> {
        let mut job_results = Vec::new();
        for (job_output, encoded) in self.job_outputs.iter().zip(outputs) {
            let EncodedJobOutput {
                feedback,
                history,
                results,
            } = encoded;
            let key = job_feedback_key(self.service, &job_output.job_type);
            let sent: Result<(), SendError> = async {
                let payload = Bytes::from(feedback?);
                if job_output.feedback_latest.borrow().as_ref() == Some(&payload) {
                    return Ok(());
                }
                let encoding = cdr_encoding(JobFeedbackList::SCHEMA_NAME);
                let sample = Sample::new(key.as_str(), Bytes::clone(&payload), &encoding);
                self.backend.publish(sample).await?;
                job_output.feedback_latest.send_replace(Some(payload));
                Ok(())
            }
            .await;
            warn_on_failure("State", &key, sent);
            match history {
                Ok(payload) => {
                    job_output.history.send_replace(Some(Bytes::from(payload)));
                }
                Err(error) => warn_on_failure(
                    "Query",
                    &job_history_key(self.service, &job_output.job_type),
                    Err(error.into()),
                ),
            }
            job_results.push(results);
        }
        job_results
    }

    pub(crate) async fn publish_job_results(
        &self,
        job_results: Vec<Vec<Result<Vec<u8>, IdlError>>>,
    ) {
        let encoding = cdr_encoding(JobResult::SCHEMA_NAME);
        for (job_output, results) in self.job_outputs.iter().zip(job_results) {
            let key = job_result_key(self.service, &job_output.job_type);
            for encoded in results {
                let sent: Result<(), SendError> = async {
                    let sample = Sample::new(key.as_str(), encoded?, encoding.as_str());
                    self.backend.publish(sample).await?;
                    Ok(())
                }
                .await;
                warn_on_failure("Event", &key, sent);
            }
        }
    }

    pub(crate) async fn publish_states(&self, encoded_states: Vec<Result<Vec<u8>, IdlError>>) {
        for (state, encoded) in self.states.iter().zip(encoded_states) {
            let sent: Result<(), SendError> = async {
                let payload = Bytes::from(encoded?);
                if state.latest.borrow().as_ref() == Some(&payload) {
                    return Ok(());
                }
                let encoding = state.endpoint.encoding.as_str();
                let sample = Sample::new(state.key.as_str(), Bytes::clone(&payload), encoding);
                self.backend.publish(sample).await?;
                state.latest.send_replace(Some(payload));
                Ok(())
            }
            .await;
            warn_on_failure("State", &state.key, sent);
        }
    }

    pub(crate) async fn publish_events(&self, events: Vec<D::Event>) {
        for event in events {
            for endpoint in &self.events {
                if let Some(encoded) = (endpoint.select)(&event) {
                    let key = event_key(self.service, &endpoint.name);
                    let sent: Result<(), SendError> = async {
                        let sample =
                            Sample::new(key.as_str(), encoded?, endpoint.encoding.as_str());
                        self.backend.publish(sample).await?;
                        Ok(())
                    }
                    .await;
                    warn_on_failure("Event", &key, sent);
                }
            }
        }
    }
}
