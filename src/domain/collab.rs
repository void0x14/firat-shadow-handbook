use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollabSnapshot {
    pub courses: Vec<CourseEntry>,
    pub playbacks: Vec<PlaybackEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CourseEntry {
    pub course_id: Option<String>,
    pub title: String,
    pub instructor: Option<String>,
    pub schedule: Option<CourseSchedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CourseSchedule {
    pub start_iso: Option<String>,
    pub end_iso: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaybackEntry {
    pub course_title: Option<String>,
    pub url: String,
    pub label: Option<String>,
}
