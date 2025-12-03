//! Scene Analysis Engine
//!
//! High-level scene analysis combining surface, lighting, and semantic data.

use nalgebra::{Point3, Vector3};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{
    SceneId, SceneState, SceneLighting, SceneBounds,
    Surface, SurfaceType, SceneObject, ObjectCategory, SemanticLabel,
    Ray, RaycastHit, PlacementCandidate,
};

/// Scene analyzer for high-level understanding
#[derive(Debug)]
pub struct SceneAnalyzer {
    config: AnalyzerConfig,
    room_classification: Option<RoomType>,
    spatial_relationships: Vec<SpatialRelation>,
    activity_zones: Vec<ActivityZone>,
    last_analysis: Instant,
}

/// Analyzer configuration
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Update interval for full analysis
    pub analysis_interval: Duration,
    /// Enable room classification
    pub classify_room: bool,
    /// Enable spatial relationship detection
    pub detect_relationships: bool,
    /// Enable activity zone detection
    pub detect_zones: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            analysis_interval: Duration::from_secs(1),
            classify_room: true,
            detect_relationships: true,
            detect_zones: true,
        }
    }
}

impl SceneAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            config,
            room_classification: None,
            spatial_relationships: Vec::new(),
            activity_zones: Vec::new(),
            last_analysis: Instant::now(),
        }
    }
    
    /// Perform full scene analysis
    pub fn analyze(&mut self, state: &SceneState) -> SceneAnalysis {
        let start = Instant::now();
        
        // Room classification
        if self.config.classify_room {
            self.room_classification = self.classify_room(state);
        }
        
        // Spatial relationships
        if self.config.detect_relationships {
            self.spatial_relationships = self.detect_spatial_relationships(&state.objects);
        }
        
        // Activity zones
        if self.config.detect_zones {
            self.activity_zones = self.detect_activity_zones(state);
        }
        
        self.last_analysis = Instant::now();
        
        SceneAnalysis {
            room_type: self.room_classification,
            spatial_relationships: self.spatial_relationships.clone(),
            activity_zones: self.activity_zones.clone(),
            metrics: SceneMetrics {
                surface_count: state.surfaces.len(),
                object_count: state.objects.len(),
                total_surface_area: state.surfaces.iter().map(|s| s.area).sum(),
                scene_volume: state.bounds.map(|b| b.volume()).unwrap_or(0.0),
                lighting_quality: self.assess_lighting_quality(&state.lighting),
            },
            analysis_time: start.elapsed(),
        }
    }
    
    fn classify_room(&self, state: &SceneState) -> Option<RoomType> {
        // Simple heuristics based on detected objects
        let object_types: Vec<&SemanticLabel> = state.objects.iter().map(|o| &o.label).collect();
        
        // Count furniture types
        let has_bed = object_types.iter().any(|l| matches!(l, SemanticLabel::Bed));
        let has_couch = object_types.iter().any(|l| matches!(l, SemanticLabel::Couch));
        let has_desk = object_types.iter().any(|l| matches!(l, SemanticLabel::Desk));
        let has_table = object_types.iter().any(|l| matches!(l, SemanticLabel::Table));
        let has_monitor = object_types.iter().any(|l| matches!(l, SemanticLabel::Monitor | SemanticLabel::Laptop));
        
        // Classify based on furniture
        if has_bed {
            Some(RoomType::Bedroom)
        } else if has_couch && has_table {
            Some(RoomType::LivingRoom)
        } else if has_desk && has_monitor {
            Some(RoomType::Office)
        } else if has_table && !has_couch {
            Some(RoomType::DiningRoom)
        } else {
            Some(RoomType::Unknown)
        }
    }
    
    fn detect_spatial_relationships(&self, objects: &[SceneObject]) -> Vec<SpatialRelation> {
        let mut relationships = Vec::new();
        
        for (i, obj_a) in objects.iter().enumerate() {
            for obj_b in objects.iter().skip(i + 1) {
                if let (Some(pos_a), Some(pos_b)) = (obj_a.position_3d, obj_b.position_3d) {
                    let relation = self.determine_relationship(&pos_a, &pos_b, obj_a, obj_b);
                    if let Some(rel) = relation {
                        relationships.push(SpatialRelation {
                            subject_id: obj_a.id,
                            object_id: obj_b.id,
                            relation: rel,
                            distance: (pos_b - pos_a).norm(),
                        });
                    }
                }
            }
        }
        
        relationships
    }
    
    fn determine_relationship(
        &self,
        pos_a: &Point3<f32>,
        pos_b: &Point3<f32>,
        _obj_a: &SceneObject,
        _obj_b: &SceneObject,
    ) -> Option<RelationType> {
        let diff = pos_b - pos_a;
        let distance = diff.norm();
        
        // Close proximity
        if distance < 0.5 {
            if diff.y > 0.3 {
                return Some(RelationType::Above);
            } else if diff.y < -0.3 {
                return Some(RelationType::Below);
            } else {
                return Some(RelationType::NextTo);
            }
        }
        
        // Medium range
        if distance < 2.0 {
            return Some(RelationType::Near);
        }
        
        None
    }
    
    fn detect_activity_zones(&self, state: &SceneState) -> Vec<ActivityZone> {
        let mut zones = Vec::new();
        
        // Detect work zones (near monitors/desks)
        for obj in &state.objects {
            if matches!(obj.label, SemanticLabel::Desk | SemanticLabel::Monitor) {
                if let Some(pos) = obj.position_3d {
                    zones.push(ActivityZone {
                        zone_type: ZoneType::Work,
                        center: pos,
                        radius: 1.5,
                        confidence: obj.confidence,
                    });
                }
            }
        }
        
        // Detect seating zones (near chairs/couches)
        for obj in &state.objects {
            if matches!(obj.label, SemanticLabel::Chair | SemanticLabel::Couch) {
                if let Some(pos) = obj.position_3d {
                    zones.push(ActivityZone {
                        zone_type: ZoneType::Seating,
                        center: pos,
                        radius: 1.0,
                        confidence: obj.confidence,
                    });
                }
            }
        }
        
        // Detect walkable zones from horizontal surfaces
        for surface in &state.surfaces {
            if surface.surface_type == SurfaceType::Horizontal && surface.area > 2.0 {
                zones.push(ActivityZone {
                    zone_type: ZoneType::Walkable,
                    center: surface.center,
                    radius: (surface.area / std::f32::consts::PI).sqrt(),
                    confidence: surface.confidence,
                });
            }
        }
        
        zones
    }
    
    fn assess_lighting_quality(&self, lighting: &SceneLighting) -> f32 {
        // Score based on brightness and color temperature
        let brightness_score = if lighting.brightness > 0.3 && lighting.brightness < 0.8 {
            1.0
        } else {
            0.5
        };
        
        let temp_score = if lighting.color_temperature > 4000.0 && lighting.color_temperature < 6500.0 {
            1.0
        } else {
            0.7
        };
        
        let main_light_score = if lighting.main_light.is_some() { 1.0 } else { 0.6 };
        
        (brightness_score + temp_score + main_light_score) / 3.0
    }
    
    /// Find best placement for content
    pub fn find_best_placement(
        &self,
        state: &SceneState,
        content_size: Vector3<f32>,
        preference: PlacementPreference,
    ) -> Vec<PlacementCandidate> {
        let mut candidates: Vec<PlacementCandidate> = Vec::new();
        let required_area = content_size.x * content_size.z;
        
        // Filter suitable surfaces
        for surface in &state.surfaces {
            let suitable = match preference {
                PlacementPreference::Floor => {
                    surface.surface_type == SurfaceType::Horizontal && 
                    surface.center.y < 0.5
                }
                PlacementPreference::Table => {
                    surface.surface_type == SurfaceType::Horizontal &&
                    surface.center.y > 0.5 && surface.center.y < 1.5
                }
                PlacementPreference::Wall => {
                    surface.surface_type == SurfaceType::Vertical
                }
                PlacementPreference::Any => {
                    surface.area >= required_area
                }
            };
            
            if suitable && surface.area >= required_area {
                let score = self.score_placement(surface, &preference, &state.lighting);
                candidates.push(PlacementCandidate {
                    surface_id: surface.id,
                    position: surface.center,
                    normal: surface.plane.normal,
                    score,
                });
            }
        }
        
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }
    
    fn score_placement(&self, surface: &Surface, preference: &PlacementPreference, lighting: &SceneLighting) -> f32 {
        let mut score = surface.confidence;
        
        // Prefer surfaces matching preference
        match preference {
            PlacementPreference::Floor if surface.surface_type == SurfaceType::Horizontal => {
                score += 0.2;
            }
            PlacementPreference::Table if surface.surface_type == SurfaceType::Horizontal => {
                score += 0.3; // Tables are often best for AR content
            }
            PlacementPreference::Wall if surface.surface_type == SurfaceType::Vertical => {
                score += 0.2;
            }
            _ => {}
        }
        
        // Bonus for good lighting
        score += lighting.brightness * 0.1;
        
        score.clamp(0.0, 1.0)
    }
    
    /// Get current room classification
    pub fn room_type(&self) -> Option<RoomType> {
        self.room_classification
    }
    
    /// Get spatial relationships
    pub fn relationships(&self) -> &[SpatialRelation] {
        &self.spatial_relationships
    }
    
    /// Get activity zones
    pub fn zones(&self) -> &[ActivityZone] {
        &self.activity_zones
    }
}

/// Result of scene analysis
#[derive(Debug, Clone)]
pub struct SceneAnalysis {
    pub room_type: Option<RoomType>,
    pub spatial_relationships: Vec<SpatialRelation>,
    pub activity_zones: Vec<ActivityZone>,
    pub metrics: SceneMetrics,
    pub analysis_time: Duration,
}

/// Scene metrics
#[derive(Debug, Clone)]
pub struct SceneMetrics {
    pub surface_count: usize,
    pub object_count: usize,
    pub total_surface_area: f32,
    pub scene_volume: f32,
    pub lighting_quality: f32,
}

/// Room type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    LivingRoom,
    Bedroom,
    Kitchen,
    Bathroom,
    Office,
    DiningRoom,
    Hallway,
    Outdoor,
    Unknown,
}

impl RoomType {
    pub fn name(&self) -> &str {
        match self {
            RoomType::LivingRoom => "Living Room",
            RoomType::Bedroom => "Bedroom",
            RoomType::Kitchen => "Kitchen",
            RoomType::Bathroom => "Bathroom",
            RoomType::Office => "Office",
            RoomType::DiningRoom => "Dining Room",
            RoomType::Hallway => "Hallway",
            RoomType::Outdoor => "Outdoor",
            RoomType::Unknown => "Unknown",
        }
    }
}

/// Spatial relationship between objects
#[derive(Debug, Clone)]
pub struct SpatialRelation {
    pub subject_id: SceneId,
    pub object_id: SceneId,
    pub relation: RelationType,
    pub distance: f32,
}

/// Types of spatial relations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    Above,
    Below,
    LeftOf,
    RightOf,
    InFrontOf,
    Behind,
    NextTo,
    Near,
    On,
    Inside,
}

impl RelationType {
    pub fn inverse(&self) -> Self {
        match self {
            RelationType::Above => RelationType::Below,
            RelationType::Below => RelationType::Above,
            RelationType::LeftOf => RelationType::RightOf,
            RelationType::RightOf => RelationType::LeftOf,
            RelationType::InFrontOf => RelationType::Behind,
            RelationType::Behind => RelationType::InFrontOf,
            RelationType::NextTo => RelationType::NextTo,
            RelationType::Near => RelationType::Near,
            RelationType::On => RelationType::Below,
            RelationType::Inside => RelationType::Inside,
        }
    }
}

/// Activity zone in the scene
#[derive(Debug, Clone)]
pub struct ActivityZone {
    pub zone_type: ZoneType,
    pub center: Point3<f32>,
    pub radius: f32,
    pub confidence: f32,
}

impl ActivityZone {
    pub fn contains(&self, point: &Point3<f32>) -> bool {
        (point - self.center).norm() <= self.radius
    }
}

/// Types of activity zones
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    Work,
    Seating,
    Dining,
    Sleeping,
    Walkable,
    Storage,
    Entertainment,
}

/// Placement preference for AR content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementPreference {
    Floor,
    Table,
    Wall,
    Any,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyzer_creation() {
        let analyzer = SceneAnalyzer::new(AnalyzerConfig::default());
        assert!(analyzer.room_type().is_none());
        assert!(analyzer.relationships().is_empty());
    }
    
    #[test]
    fn test_analyzer_config_default() {
        let config = AnalyzerConfig::default();
        assert!(config.classify_room);
        assert!(config.detect_relationships);
        assert!(config.detect_zones);
    }
    
    #[test]
    fn test_room_type_name() {
        assert_eq!(RoomType::LivingRoom.name(), "Living Room");
        assert_eq!(RoomType::Office.name(), "Office");
        assert_eq!(RoomType::Unknown.name(), "Unknown");
    }
    
    #[test]
    fn test_relation_type_inverse() {
        assert_eq!(RelationType::Above.inverse(), RelationType::Below);
        assert_eq!(RelationType::Below.inverse(), RelationType::Above);
        assert_eq!(RelationType::NextTo.inverse(), RelationType::NextTo);
    }
    
    #[test]
    fn test_activity_zone_contains() {
        let zone = ActivityZone {
            zone_type: ZoneType::Work,
            center: Point3::new(0.0, 0.0, 0.0),
            radius: 1.0,
            confidence: 0.8,
        };
        
        assert!(zone.contains(&Point3::new(0.5, 0.0, 0.0)));
        assert!(!zone.contains(&Point3::new(2.0, 0.0, 0.0)));
    }
    
    #[test]
    fn test_analyze_empty_scene() {
        let mut analyzer = SceneAnalyzer::new(AnalyzerConfig::default());
        let state = SceneState::default();
        
        let analysis = analyzer.analyze(&state);
        
        assert_eq!(analysis.metrics.surface_count, 0);
        assert_eq!(analysis.metrics.object_count, 0);
    }
    
    #[test]
    fn test_scene_metrics() {
        let metrics = SceneMetrics {
            surface_count: 5,
            object_count: 10,
            total_surface_area: 25.0,
            scene_volume: 50.0,
            lighting_quality: 0.8,
        };
        
        assert_eq!(metrics.surface_count, 5);
        assert_eq!(metrics.object_count, 10);
    }
}
