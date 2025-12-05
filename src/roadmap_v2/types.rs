// src/roadmap_v2/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskStore {
    pub meta: RoadmapMeta,
    #[serde(default)]
    pub sections: Vec<Section>,
    #[serde(default)]
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoadmapMeta {
    pub title: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub status: SectionStatus,
    #[serde(default)]
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SectionStatus {
    #[default]
    Pending,
    Current,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub status: TaskStatus,
    pub section: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub test: Option<String>,
    #[serde(default)]
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    #[default]
    Pending,
    Done,
    NoTest,
}

#[derive(Debug, Clone)]
pub enum RoadmapCommand {
    Check { id: String },
    Uncheck { id: String },
    Add(Task),
    Update { id: String, fields: TaskUpdate },
    Delete { id: String },
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskUpdate {
    pub text: Option<String>,
    pub test: Option<String>,
    pub section: Option<String>,
    pub group: Option<String>,
}