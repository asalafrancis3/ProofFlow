/// Event decoder for ProofFlow contract events.
///
/// Each event emitted by the contract has a topic (symbol) and data payload.
/// This module decodes raw Soroban event data into typed `IndexEvent` variants.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndexEvent {
    JobCreated {
        job_id: u64,
        client: String,
    },
    JobFunded {
        job_id: u64,
    },
    JobActivated {
        job_id: u64,
    },
    JobCancelled {
        job_id: u64,
    },
    JobSettled {
        job_id: u64,
    },
    MilestoneSubmitted {
        job_id: u64,
        index: u32,
        worker: String,
    },
    MilestoneApproved {
        job_id: u64,
        index: u32,
    },
    MilestoneRejected {
        job_id: u64,
        index: u32,
    },
    MilestoneReleased {
        job_id: u64,
        index: u32,
    },
    EscrowFunded {
        job_id: u64,
        amount: u128,
    },
    EscrowCreated {
        job_id: u64,
    },
    EscrowFrozen {
        job_id: u64,
        dispute_id: u32,
    },
    EscrowUnfrozen {
        job_id: u64,
        dispute_id: u32,
    },
    EscrowReleased {
        job_id: u64,
        milestone_idx: u32,
        amount: u128,
    },
    MilestoneCreated {
        job_id: u64,
        index: u32,
        amount: u128,
        worker: String,
    },
    DisputeFiled {
        job_id: u64,
        milestone_idx: u32,
        dispute_id: u32,
    },
    DisputeResolved {
        job_id: u64,
        dispute_id: u32,
    },
    ReputationUpdated {
        address: String,
        old_score: u64,
        new_score: u64,
    },
    UserRegistered {
        address: String,
        role: String,
    },
    VerifierAdded {
        address: String,
    },
    VerifierRemoved {
        address: String,
    },
    Unknown {
        topic: String,
        raw_data: String,
    },
}

pub struct ContractEventDecoder;

impl ContractEventDecoder {
    /// Decode a raw Soroban event (topic + data) into an `IndexEvent`.
    ///
    /// `topic` is the event symbol (e.g., "JOB_CR", "MS_SUB").
    /// `data` is the JSON-encoded event payload.
    pub fn decode(topic: &str, data: &str) -> IndexEvent {
        match topic {
            "JOB_CR" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::JobCreated {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    client: parsed["client"].as_str().unwrap_or("").to_string(),
                }
            }
            "JOB_FND" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::JobFunded {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                }
            }
            "JOB_ACT" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::JobActivated {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                }
            }
            "JOB_CNC" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::JobCancelled {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                }
            }
            "JOB_STL" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::JobSettled {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                }
            }
            "MS_CR" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::MilestoneCreated {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    index: parsed["index"].as_u64().unwrap_or(0) as u32,
                    amount: parsed["amount"].as_u64().unwrap_or(0) as u128,
                    worker: parsed["worker"].as_str().unwrap_or("").to_string(),
                }
            }
            "MS_SUB" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::MilestoneSubmitted {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    index: parsed["index"].as_u64().unwrap_or(0) as u32,
                    worker: parsed["worker"].as_str().unwrap_or("").to_string(),
                }
            }
            "MS_APR" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::MilestoneApproved {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    index: parsed["index"].as_u64().unwrap_or(0) as u32,
                }
            }
            "MS_REJ" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::MilestoneRejected {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    index: parsed["index"].as_u64().unwrap_or(0) as u32,
                }
            }
            "MS_RLS" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::MilestoneReleased {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    index: parsed["index"].as_u64().unwrap_or(0) as u32,
                }
            }
            "ESC_FND" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::EscrowFunded {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    amount: parsed["amount"].as_u64().unwrap_or(0) as u128,
                }
            }
            "ESC_CR" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::EscrowCreated {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                }
            }
            "ESC_FRZ" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::EscrowFrozen {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    dispute_id: parsed["dispute_id"].as_u64().unwrap_or(0) as u32,
                }
            }
            "ESC_UNF" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::EscrowUnfrozen {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    dispute_id: parsed["dispute_id"].as_u64().unwrap_or(0) as u32,
                }
            }
            "ESC_RLS" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::EscrowReleased {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    milestone_idx: parsed["milestone_idx"].as_u64().unwrap_or(0) as u32,
                    amount: parsed["amount"].as_u64().unwrap_or(0) as u128,
                }
            }
            "DISP_FL" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::DisputeFiled {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    milestone_idx: parsed["milestone_idx"].as_u64().unwrap_or(0) as u32,
                    dispute_id: parsed["dispute_id"].as_u64().unwrap_or(0) as u32,
                }
            }
            "DISP_RS" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::DisputeResolved {
                    job_id: parsed["job_id"].as_u64().unwrap_or(0),
                    dispute_id: parsed["dispute_id"].as_u64().unwrap_or(0) as u32,
                }
            }
            "USR_REG" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::UserRegistered {
                    address: parsed["address"].as_str().unwrap_or("").to_string(),
                    role: parsed["role"].as_str().unwrap_or("").to_string(),
                }
            }
            "REP_UPD" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::ReputationUpdated {
                    address: parsed["address"].as_str().unwrap_or("").to_string(),
                    old_score: parsed["old_score"].as_u64().unwrap_or(0),
                    new_score: parsed["new_score"].as_u64().unwrap_or(0),
                }
            }
            "VER_ADD" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::VerifierAdded {
                    address: parsed["address"].as_str().unwrap_or("").to_string(),
                }
            }
            "VER_REM" => {
                let parsed: serde_json::Value = serde_json::from_str(data)
                    .unwrap_or(serde_json::Value::Null);
                IndexEvent::VerifierRemoved {
                    address: parsed["address"].as_str().unwrap_or("").to_string(),
                }
            }
            _ => IndexEvent::Unknown {
                topic: topic.to_string(),
                raw_data: data.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_job_created() {
        let event = ContractEventDecoder::decode(
            "JOB_CR",
            r#"{"job_id":42,"client":"CAAAA..."}"#,
        );
        assert_eq!(
            event,
            IndexEvent::JobCreated {
                job_id: 42,
                client: "CAAAA...".to_string(),
            }
        );
    }

    #[test]
    fn decode_milestone_submitted() {
        let event = ContractEventDecoder::decode(
            "MS_SUB",
            r#"{"job_id":1,"index":0,"worker":"CBBBB..."}"#,
        );
        assert_eq!(
            event,
            IndexEvent::MilestoneSubmitted {
                job_id: 1,
                index: 0,
                worker: "CBBBB...".to_string(),
            }
        );
    }

    #[test]
    fn decode_unknown_topic() {
        let event = ContractEventDecoder::decode("UNKNOWN_TOPIC", "{}");
        assert!(matches!(event, IndexEvent::Unknown { .. }));
    }

    #[test]
    fn decode_malformed_json() {
        let event = ContractEventDecoder::decode("JOB_CR", "not json");
        assert_eq!(
            event,
            IndexEvent::JobCreated {
                job_id: 0,
                client: "".to_string(),
            }
        );
    }
}
