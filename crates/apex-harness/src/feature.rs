use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeatureError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid feature transition: from {from:?} to {to:?}")]
    InvalidTransition { from: FeatureState, to: FeatureState },
    #[error("Feature not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureState {
    NotStarted,
    Active,
    Blocked,
    Passing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub module: String,
    pub behavior: String,
    pub verification: String,
    pub state: FeatureState,
    #[serde(default)]
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRegistry {
    pub features: Vec<Feature>,
}

impl FeatureRegistry {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, FeatureError> {
        let content = std::fs::read_to_string(path)?;
        let features: Vec<Feature> = serde_json::from_str(&content)?;
        Ok(Self { features })
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), FeatureError> {
        let content = serde_json::to_string_pretty(&self.features)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_feature(&self, id: &str) -> Option<&Feature> {
        self.features.iter().find(|f| f.id == id)
    }

    pub fn get_feature_mut(&mut self, id: &str) -> Option<&mut Feature> {
        self.features.iter_mut().find(|f| f.id == id)
    }

    /// Back-pressure: returns the count of features that are not yet passing.
    pub fn back_pressure_count(&self) -> usize {
        self.features.iter().filter(|f| f.state != FeatureState::Passing).count()
    }
}
