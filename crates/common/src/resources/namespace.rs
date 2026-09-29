use crate::types::{ObjectMeta, Phase, TypeMeta};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Namespace provides a scope for resource names
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Namespace {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub metadata: ObjectMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<NamespaceSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<NamespaceStatus>,
}

impl Namespace {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            type_meta: TypeMeta {
                kind: "Namespace".to_string(),
                api_version: "v1".to_string(),
            },
            metadata: ObjectMeta::new(name),
            spec: None,
            status: Some(NamespaceStatus {
                phase: Some(Phase::Active),
                conditions: None,
            }),
        }
    }

    /// Set the `kubernetes.io/metadata.name` label to the namespace name, as the API server
    /// does for every namespace. Returns true if the labels changed.
    pub fn ensure_name_label(&mut self) -> bool {
        let labels = self.metadata.labels.get_or_insert_with(Default::default);
        if labels.get(NAME_LABEL) == Some(&self.metadata.name) {
            return false;
        }
        labels.insert(NAME_LABEL.to_string(), self.metadata.name.clone());
        true
    }
}

/// Label the API server sets on every namespace to its own name
pub const NAME_LABEL: &str = "kubernetes.io/metadata.name";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceSpec {
    /// Finalizers is a list of finalizers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finalizers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceStatus {
    /// Phase is the current lifecycle phase of the namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<Phase>,

    /// Conditions describe the current conditions of a namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<NamespaceCondition>>,
}

/// NamespaceCondition contains details about the current condition of a namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceCondition {
    /// Type of namespace condition (NamespaceDeletionDiscoveryFailure, NamespaceDeletionGroupVersionParsingFailure, etc.)
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition (True, False, Unknown)
    pub status: String,

    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        serialize_with = "crate::time::k8s_time::serialize",
        deserialize_with = "crate::time::k8s_time::deserialize"
    )]
    pub last_transition_time: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_name_label_adds_label() {
        let mut ns = Namespace::new("kube-system");
        assert!(ns.ensure_name_label());
        assert_eq!(
            ns.metadata.labels.as_ref().unwrap().get(NAME_LABEL),
            Some(&"kube-system".to_string())
        );
    }

    #[test]
    fn test_ensure_name_label_is_idempotent() {
        let mut ns = Namespace::new("default");
        assert!(ns.ensure_name_label());
        assert!(!ns.ensure_name_label());
    }

    #[test]
    fn test_ensure_name_label_overrides_wrong_value_and_keeps_other_labels() {
        let mut ns = Namespace::new("team-a");
        let labels = ns.metadata.labels.get_or_insert_with(Default::default);
        labels.insert(NAME_LABEL.to_string(), "other".to_string());
        labels.insert("env".to_string(), "dev".to_string());

        assert!(ns.ensure_name_label());
        let labels = ns.metadata.labels.as_ref().unwrap();
        assert_eq!(labels.get(NAME_LABEL), Some(&"team-a".to_string()));
        assert_eq!(labels.get("env"), Some(&"dev".to_string()));
    }
}
