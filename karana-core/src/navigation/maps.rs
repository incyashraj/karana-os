//! Map Rendering for Kāraṇa OS AR Glasses
//!
//! Map tiles and rendering for mini-map display.

use std::collections::HashMap;
use nalgebra::Vector2;
use super::location::Location;

/// Map layer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapLayer {
    /// Base street map
    Streets,
    /// Satellite imagery
    Satellite,
    /// Terrain/topographic
    Terrain,
    /// Traffic conditions
    Traffic,
    /// Public transit
    Transit,
    /// Cycling routes
    Cycling,
    /// Buildings (3D)
    Buildings,
    /// POI markers
    POIMarkers,
}

impl MapLayer {
    /// Get display name
    pub fn name(&self) -> &str {
        match self {
            Self::Streets => "Streets",
            Self::Satellite => "Satellite",
            Self::Terrain => "Terrain",
            Self::Traffic => "Traffic",
            Self::Transit => "Transit",
            Self::Cycling => "Cycling",
            Self::Buildings => "Buildings",
            Self::POIMarkers => "POI Markers",
        }
    }
    
    /// Is overlay layer (can be combined with base)
    pub fn is_overlay(&self) -> bool {
        matches!(
            self,
            Self::Traffic | Self::Transit | Self::Cycling | Self::Buildings | Self::POIMarkers
        )
    }
}

/// Map tile coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord {
    /// X coordinate
    pub x: u32,
    /// Y coordinate
    pub y: u32,
    /// Zoom level
    pub zoom: u8,
}

impl TileCoord {
    /// Create new tile coordinate
    pub fn new(x: u32, y: u32, zoom: u8) -> Self {
        Self { x, y, zoom }
    }
    
    /// Get tile coordinate from lat/lon
    pub fn from_location(lat: f64, lon: f64, zoom: u8) -> Self {
        let n = 2.0_f64.powi(zoom as i32);
        let x = ((lon + 180.0) / 360.0 * n).floor() as u32;
        
        let lat_rad = lat.to_radians();
        let y = ((1.0 - lat_rad.tan().asinh() / std::f64::consts::PI) / 2.0 * n).floor() as u32;
        
        Self { x, y, zoom }
    }
    
    /// Get center location of tile
    pub fn center_location(&self) -> (f64, f64) {
        let n = 2.0_f64.powi(self.zoom as i32);
        let lon = (self.x as f64) / n * 360.0 - 180.0;
        
        let lat_rad = (std::f64::consts::PI * (1.0 - 2.0 * (self.y as f64) / n)).sinh().atan();
        let lat = lat_rad.to_degrees();
        
        (lat, lon)
    }
    
    /// Get neighboring tiles
    pub fn neighbors(&self) -> Vec<TileCoord> {
        let mut neighbors = Vec::with_capacity(8);
        let max = 2u32.pow(self.zoom as u32);
        
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = (self.x as i32 + dx).rem_euclid(max as i32) as u32;
                let ny = (self.y as i32 + dy).rem_euclid(max as i32) as u32;
                
                neighbors.push(TileCoord::new(nx, ny, self.zoom));
            }
        }
        
        neighbors
    }
}

/// Map tile state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileState {
    /// Not loaded
    Empty,
    /// Loading
    Loading,
    /// Loaded and ready
    Ready,
    /// Failed to load
    Failed,
}

/// Map tile
#[derive(Debug, Clone)]
pub struct MapTile {
    /// Tile coordinates
    pub coord: TileCoord,
    /// Layer
    pub layer: MapLayer,
    /// Tile state
    pub state: TileState,
    /// Texture/image data (placeholder)
    pub data: Option<Vec<u8>>,
    /// Last access time (for cache eviction)
    pub last_access: std::time::Instant,
}

impl MapTile {
    /// Create new empty tile
    pub fn new(coord: TileCoord, layer: MapLayer) -> Self {
        Self {
            coord,
            layer,
            state: TileState::Empty,
            data: None,
            last_access: std::time::Instant::now(),
        }
    }
    
    /// Mark as loading
    pub fn start_loading(&mut self) {
        self.state = TileState::Loading;
    }
    
    /// Set loaded data
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = Some(data);
        self.state = TileState::Ready;
    }
    
    /// Mark as failed
    pub fn set_failed(&mut self) {
        self.state = TileState::Failed;
    }
    
    /// Touch (update last access)
    pub fn touch(&mut self) {
        self.last_access = std::time::Instant::now();
    }
}

/// Map viewport
#[derive(Debug, Clone)]
pub struct MapViewport {
    /// Center location
    pub center: Location,
    /// Zoom level
    pub zoom: u8,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Rotation (degrees)
    pub rotation: f32,
    /// Tilt (for 3D view)
    pub tilt: f32,
}

impl MapViewport {
    /// Create new viewport
    pub fn new(center: Location, zoom: u8, width: u32, height: u32) -> Self {
        Self {
            center,
            zoom,
            width,
            height,
            rotation: 0.0,
            tilt: 0.0,
        }
    }
    
    /// Get visible tile coordinates
    pub fn visible_tiles(&self) -> Vec<TileCoord> {
        let center_tile = TileCoord::from_location(
            self.center.latitude(),
            self.center.longitude(),
            self.zoom,
        );
        
        // Calculate how many tiles fit in viewport
        let tile_size = 256.0;
        let tiles_x = (self.width as f32 / tile_size).ceil() as i32 + 1;
        let tiles_y = (self.height as f32 / tile_size).ceil() as i32 + 1;
        
        let mut tiles = Vec::new();
        let max = 2u32.pow(self.zoom as u32);
        
        for dx in -(tiles_x / 2)..=(tiles_x / 2) {
            for dy in -(tiles_y / 2)..=(tiles_y / 2) {
                let x = (center_tile.x as i32 + dx).rem_euclid(max as i32) as u32;
                let y = (center_tile.y as i32 + dy).rem_euclid(max as i32) as u32;
                
                tiles.push(TileCoord::new(x, y, self.zoom));
            }
        }
        
        tiles
    }
    
    /// Zoom in
    pub fn zoom_in(&mut self) {
        if self.zoom < 20 {
            self.zoom += 1;
        }
    }
    
    /// Zoom out
    pub fn zoom_out(&mut self) {
        if self.zoom > 1 {
            self.zoom -= 1;
        }
    }
    
    /// Pan by pixels
    pub fn pan(&mut self, dx: f32, dy: f32) {
        // Convert pixel offset to lat/lon at current zoom
        let scale = 360.0 / (256.0 * 2.0_f64.powi(self.zoom as i32));
        let dlon = (dx as f64) * scale;
        let dlat = -(dy as f64) * scale;
        
        self.center = Location::new(
            self.center.latitude() + dlat,
            self.center.longitude() + dlon,
        );
    }
}

/// Map renderer
#[derive(Debug)]
pub struct MapRenderer {
    /// Tile cache
    tiles: HashMap<(TileCoord, MapLayer), MapTile>,
    /// Active layers
    active_layers: Vec<MapLayer>,
    /// Base layer
    base_layer: MapLayer,
    /// Current viewport
    viewport: MapViewport,
    /// Cache size limit
    cache_limit: usize,
    /// Show user location
    show_user_location: bool,
    /// User location marker style
    user_marker_style: UserMarkerStyle,
}

/// User location marker style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserMarkerStyle {
    /// Simple dot
    Dot,
    /// Arrow showing direction
    Arrow,
    /// Compass with direction
    Compass,
    /// Pulsing circle
    Pulse,
}

impl MapRenderer {
    /// Create new map renderer
    pub fn new(initial_location: Location) -> Self {
        Self {
            tiles: HashMap::new(),
            active_layers: vec![MapLayer::POIMarkers],
            base_layer: MapLayer::Streets,
            viewport: MapViewport::new(initial_location, 15, 300, 200),
            cache_limit: 100,
            show_user_location: true,
            user_marker_style: UserMarkerStyle::Arrow,
        }
    }
    
    /// Set base layer
    pub fn set_base_layer(&mut self, layer: MapLayer) {
        if !layer.is_overlay() {
            self.base_layer = layer;
        }
    }
    
    /// Toggle overlay layer
    pub fn toggle_layer(&mut self, layer: MapLayer) {
        if layer.is_overlay() {
            if let Some(pos) = self.active_layers.iter().position(|l| *l == layer) {
                self.active_layers.remove(pos);
            } else {
                self.active_layers.push(layer);
            }
        }
    }
    
    /// Is layer active
    pub fn is_layer_active(&self, layer: MapLayer) -> bool {
        self.active_layers.contains(&layer) || self.base_layer == layer
    }
    
    /// Update viewport center
    pub fn set_center(&mut self, location: Location) {
        self.viewport.center = location;
    }
    
    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: u8) {
        self.viewport.zoom = zoom.clamp(1, 20);
    }
    
    /// Get current zoom
    pub fn zoom(&self) -> u8 {
        self.viewport.zoom
    }
    
    /// Zoom in
    pub fn zoom_in(&mut self) {
        self.viewport.zoom_in();
    }
    
    /// Zoom out
    pub fn zoom_out(&mut self) {
        self.viewport.zoom_out();
    }
    
    /// Pan map
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.viewport.pan(dx, dy);
    }
    
    /// Get visible tiles
    pub fn get_visible_tiles(&self) -> Vec<TileCoord> {
        self.viewport.visible_tiles()
    }
    
    /// Request tile
    pub fn request_tile(&mut self, coord: TileCoord, layer: MapLayer) {
        let key = (coord, layer);
        if !self.tiles.contains_key(&key) {
            let mut tile = MapTile::new(coord, layer);
            tile.start_loading();
            self.tiles.insert(key, tile);
            
            // In real implementation, would trigger async load
        }
    }
    
    /// Get tile if loaded
    pub fn get_tile(&mut self, coord: TileCoord, layer: MapLayer) -> Option<&MapTile> {
        let key = (coord, layer);
        if let Some(tile) = self.tiles.get_mut(&key) {
            tile.touch();
            Some(tile)
        } else {
            None
        }
    }
    
    /// Set tile data
    pub fn set_tile_data(&mut self, coord: TileCoord, layer: MapLayer, data: Vec<u8>) {
        let key = (coord, layer);
        if let Some(tile) = self.tiles.get_mut(&key) {
            tile.set_data(data);
        }
    }
    
    /// Evict old tiles if over cache limit
    pub fn evict_old_tiles(&mut self) {
        if self.tiles.len() <= self.cache_limit {
            return;
        }
        
        let mut tiles: Vec<_> = self.tiles.keys().cloned().collect();
        tiles.sort_by_key(|k| {
            self.tiles.get(k)
                .map(|t| t.last_access)
                .unwrap_or(std::time::Instant::now())
        });
        
        let to_remove = tiles.len() - self.cache_limit;
        for key in tiles.into_iter().take(to_remove) {
            self.tiles.remove(&key);
        }
    }
    
    /// Clear tile cache
    pub fn clear_cache(&mut self) {
        self.tiles.clear();
    }
    
    /// Set user marker style
    pub fn set_user_marker_style(&mut self, style: UserMarkerStyle) {
        self.user_marker_style = style;
    }
    
    /// Toggle user location display
    pub fn set_show_user_location(&mut self, show: bool) {
        self.show_user_location = show;
    }
    
    /// Get viewport
    pub fn viewport(&self) -> &MapViewport {
        &self.viewport
    }
}

impl Default for MapRenderer {
    fn default() -> Self {
        Self::new(Location::new(0.0, 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_map_layer() {
        assert!(!MapLayer::Streets.is_overlay());
        assert!(MapLayer::Traffic.is_overlay());
    }
    
    #[test]
    fn test_tile_coord_from_location() {
        let coord = TileCoord::from_location(40.7128, -74.0060, 10);
        assert!(coord.x > 0);
        assert!(coord.y > 0);
    }
    
    #[test]
    fn test_tile_neighbors() {
        let coord = TileCoord::new(100, 100, 10);
        let neighbors = coord.neighbors();
        assert_eq!(neighbors.len(), 8);
    }
    
    #[test]
    fn test_map_tile() {
        let coord = TileCoord::new(0, 0, 10);
        let mut tile = MapTile::new(coord, MapLayer::Streets);
        
        assert_eq!(tile.state, TileState::Empty);
        
        tile.start_loading();
        assert_eq!(tile.state, TileState::Loading);
        
        tile.set_data(vec![1, 2, 3]);
        assert_eq!(tile.state, TileState::Ready);
    }
    
    #[test]
    fn test_map_viewport() {
        let location = Location::new(40.7128, -74.0060);
        let mut viewport = MapViewport::new(location, 15, 300, 200);
        
        viewport.zoom_in();
        assert_eq!(viewport.zoom, 16);
        
        viewport.zoom_out();
        assert_eq!(viewport.zoom, 15);
    }
    
    #[test]
    fn test_visible_tiles() {
        let location = Location::new(40.7128, -74.0060);
        let viewport = MapViewport::new(location, 15, 300, 200);
        
        let tiles = viewport.visible_tiles();
        assert!(!tiles.is_empty());
    }
    
    #[test]
    fn test_map_renderer_creation() {
        let location = Location::new(40.7128, -74.0060);
        let renderer = MapRenderer::new(location);
        
        assert_eq!(renderer.zoom(), 15);
    }
    
    #[test]
    fn test_toggle_layer() {
        let location = Location::new(40.7128, -74.0060);
        let mut renderer = MapRenderer::new(location);
        
        assert!(!renderer.is_layer_active(MapLayer::Traffic));
        
        renderer.toggle_layer(MapLayer::Traffic);
        assert!(renderer.is_layer_active(MapLayer::Traffic));
        
        renderer.toggle_layer(MapLayer::Traffic);
        assert!(!renderer.is_layer_active(MapLayer::Traffic));
    }
    
    #[test]
    fn test_set_base_layer() {
        let location = Location::new(40.7128, -74.0060);
        let mut renderer = MapRenderer::new(location);
        
        renderer.set_base_layer(MapLayer::Satellite);
        assert!(renderer.is_layer_active(MapLayer::Satellite));
    }
}
