//! UI Layout System
//! 
//! Flexible layout managers for AR interfaces including:
//! - Flexbox layout
//! - Grid layout
//! - Stack layout
//! - Absolute positioning

use super::*;
use std::any::Any;

/// Main axis for flex layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// How children wrap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Main axis alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross axis alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    #[default]
    Center,
    Stretch,
    Baseline,
}

/// Cross axis alignment for multi-line content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

// =============================================================================
// Flex Container
// =============================================================================

/// Flexbox layout container
pub struct Flex {
    id: u64,
    bounds: Rect,
    children: Vec<Box<dyn Widget>>,
    direction: FlexDirection,
    wrap: FlexWrap,
    justify: JustifyContent,
    align_items: AlignItems,
    align_content: AlignContent,
    gap: f32,
    padding: EdgeInsets,
    visible: bool,
    child_sizes: Vec<Size>,
}

impl Flex {
    pub fn new() -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            children: Vec::new(),
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            align_content: AlignContent::FlexStart,
            gap: 0.0,
            padding: EdgeInsets::default(),
            visible: true,
            child_sizes: Vec::new(),
        }
    }

    pub fn row() -> Self {
        Self::new().with_direction(FlexDirection::Row)
    }

    pub fn column() -> Self {
        Self::new().with_direction(FlexDirection::Column)
    }

    pub fn with_direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_wrap(mut self, wrap: FlexWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn with_justify(mut self, justify: JustifyContent) -> Self {
        self.justify = justify;
        self
    }

    pub fn with_align_items(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    pub fn add_child(&mut self, child: Box<dyn Widget>) {
        self.children.push(child);
    }

    fn is_horizontal(&self) -> bool {
        matches!(self.direction, FlexDirection::Row | FlexDirection::RowReverse)
    }

    fn is_reversed(&self) -> bool {
        matches!(self.direction, FlexDirection::RowReverse | FlexDirection::ColumnReverse)
    }
}

impl Default for Flex {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Flex {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Flex" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let available_width = constraints.max_width - self.padding.horizontal();
        let available_height = constraints.max_height - self.padding.vertical();

        // First pass: measure all children
        self.child_sizes.clear();
        let child_constraints = LayoutConstraints::loose(Size::new(
            if self.is_horizontal() { f32::INFINITY } else { available_width },
            if self.is_horizontal() { available_height } else { f32::INFINITY },
        ));

        for child in &mut self.children {
            let size = child.layout(&child_constraints)?;
            self.child_sizes.push(size);
        }

        // Calculate total size
        let total_gap = self.gap * (self.children.len().saturating_sub(1)) as f32;
        let (main_total, cross_max) = if self.is_horizontal() {
            let main: f32 = self.child_sizes.iter().map(|s| s.width).sum();
            let cross = self.child_sizes.iter().map(|s| s.height).fold(0.0f32, |a, b| a.max(b));
            (main + total_gap, cross)
        } else {
            let main: f32 = self.child_sizes.iter().map(|s| s.height).sum();
            let cross = self.child_sizes.iter().map(|s| s.width).fold(0.0f32, |a, b| a.max(b));
            (main + total_gap, cross)
        };

        let content_size = if self.is_horizontal() {
            Size::new(main_total, cross_max)
        } else {
            Size::new(cross_max, main_total)
        };

        self.bounds.size = constraints.constrain(Size::new(
            content_size.width + self.padding.horizontal(),
            content_size.height + self.padding.vertical(),
        ));
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;

        if self.children.is_empty() {
            return;
        }

        let content_x = position.x + self.padding.left;
        let content_y = position.y + self.padding.top;
        let content_width = self.bounds.width() - self.padding.horizontal();
        let content_height = self.bounds.height() - self.padding.vertical();

        let total_gap = self.gap * (self.children.len().saturating_sub(1)) as f32;
        let (main_total, _) = if self.is_horizontal() {
            (self.child_sizes.iter().map(|s| s.width).sum::<f32>() + total_gap, content_height)
        } else {
            (self.child_sizes.iter().map(|s| s.height).sum::<f32>() + total_gap, content_width)
        };

        let main_space = if self.is_horizontal() { content_width } else { content_height };
        let extra_space = (main_space - main_total).max(0.0);

        // Calculate starting position and gaps based on justify
        let (mut main_pos, gap_between, gap_around) = match self.justify {
            JustifyContent::FlexStart => (0.0, self.gap, 0.0),
            JustifyContent::FlexEnd => (extra_space, self.gap, 0.0),
            JustifyContent::Center => (extra_space / 2.0, self.gap, 0.0),
            JustifyContent::SpaceBetween => {
                let gap = if self.children.len() > 1 {
                    extra_space / (self.children.len() - 1) as f32
                } else {
                    0.0
                };
                (0.0, gap, 0.0)
            }
            JustifyContent::SpaceAround => {
                let gap = extra_space / (self.children.len() * 2) as f32;
                (gap, self.gap + gap * 2.0, gap)
            }
            JustifyContent::SpaceEvenly => {
                let gap = extra_space / (self.children.len() + 1) as f32;
                (gap, self.gap + gap, gap)
            }
        };

        main_pos += gap_around;

        // Pre-compute values to avoid borrow conflicts
        let is_horizontal = self.is_horizontal();
        let align_items = self.align_items;
        let num_indices = self.children.len();

        // Position children
        let indices: Vec<usize> = if self.is_reversed() {
            (0..num_indices).rev().collect()
        } else {
            (0..num_indices).collect()
        };

        for (i, &idx) in indices.iter().enumerate() {
            let size = self.child_sizes[idx];

            // Cross axis alignment
            let cross_pos = match align_items {
                AlignItems::FlexStart => 0.0,
                AlignItems::FlexEnd => {
                    if is_horizontal {
                        content_height - size.height
                    } else {
                        content_width - size.width
                    }
                }
                AlignItems::Center => {
                    if is_horizontal {
                        (content_height - size.height) / 2.0
                    } else {
                        (content_width - size.width) / 2.0
                    }
                }
                AlignItems::Stretch | AlignItems::Baseline => 0.0,
            };

            let child_pos = if is_horizontal {
                Point::new(content_x + main_pos, content_y + cross_pos)
            } else {
                Point::new(content_x + cross_pos, content_y + main_pos)
            };

            self.children[idx].set_position(child_pos);

            // Move main position
            let main_size = if is_horizontal { size.width } else { size.height };
            main_pos += main_size;
            if i < indices.len() - 1 {
                main_pos += gap_between;
            }
        }
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, ctx: &UiContext) -> Result<(), UiError> {
        for child in &mut self.children {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &UiContext) -> bool {
        for child in &mut self.children {
            if child.handle_event(event, ctx) {
                return true;
            }
        }
        false
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError> {
        for child in &self.children {
            child.render(ctx)?;
        }
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        &mut self.children
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Stack Layout
// =============================================================================

/// Overlapping children layout (z-stack)
pub struct Stack {
    id: u64,
    bounds: Rect,
    children: Vec<Box<dyn Widget>>,
    alignment: Alignment,
    visible: bool,
}

/// 2D alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    #[default]
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Alignment {
    pub fn offset_for_size(&self, container: Size, child: Size) -> Point {
        let x = match self {
            Alignment::TopLeft | Alignment::CenterLeft | Alignment::BottomLeft => 0.0,
            Alignment::TopCenter | Alignment::Center | Alignment::BottomCenter => {
                (container.width - child.width) / 2.0
            }
            Alignment::TopRight | Alignment::CenterRight | Alignment::BottomRight => {
                container.width - child.width
            }
        };

        let y = match self {
            Alignment::TopLeft | Alignment::TopCenter | Alignment::TopRight => 0.0,
            Alignment::CenterLeft | Alignment::Center | Alignment::CenterRight => {
                (container.height - child.height) / 2.0
            }
            Alignment::BottomLeft | Alignment::BottomCenter | Alignment::BottomRight => {
                container.height - child.height
            }
        };

        Point::new(x, y)
    }
}

impl Stack {
    pub fn new() -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            children: Vec::new(),
            alignment: Alignment::Center,
            visible: true,
        }
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    pub fn add_child(&mut self, child: Box<dyn Widget>) {
        self.children.push(child);
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Stack {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Stack" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let mut max_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;

        for child in &mut self.children {
            let size = child.layout(constraints)?;
            max_width = max_width.max(size.width);
            max_height = max_height.max(size.height);
        }

        self.bounds.size = constraints.constrain(Size::new(max_width, max_height));
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;

        for child in &mut self.children {
            let child_bounds = child.bounds();
            let offset = self.alignment.offset_for_size(self.bounds.size, child_bounds.size);
            child.set_position(Point::new(position.x + offset.x, position.y + offset.y));
        }
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, ctx: &UiContext) -> Result<(), UiError> {
        for child in &mut self.children {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &UiContext) -> bool {
        // Process in reverse order (top first)
        for child in self.children.iter_mut().rev() {
            if child.handle_event(event, ctx) {
                return true;
            }
        }
        false
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError> {
        for child in &self.children {
            child.render(ctx)?;
        }
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn children(&self) -> &[Box<dyn Widget>] { &self.children }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { &mut self.children }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Grid Layout
// =============================================================================

/// Grid layout configuration
#[derive(Debug, Clone)]
pub struct GridConfig {
    pub columns: usize,
    pub row_gap: f32,
    pub column_gap: f32,
    pub auto_rows: GridTrack,
    pub auto_columns: GridTrack,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            columns: 2,
            row_gap: 8.0,
            column_gap: 8.0,
            auto_rows: GridTrack::Auto,
            auto_columns: GridTrack::Fr(1.0),
        }
    }
}

/// Grid track sizing
#[derive(Debug, Clone, Copy)]
pub enum GridTrack {
    Auto,
    Fixed(f32),
    Fr(f32),
    MinMax(f32, f32),
}

/// Grid layout
pub struct Grid {
    id: u64,
    bounds: Rect,
    children: Vec<Box<dyn Widget>>,
    config: GridConfig,
    padding: EdgeInsets,
    visible: bool,
    child_rects: Vec<Rect>,
}

impl Grid {
    pub fn new(columns: usize) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            children: Vec::new(),
            config: GridConfig { columns, ..Default::default() },
            padding: EdgeInsets::default(),
            visible: true,
            child_rects: Vec::new(),
        }
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.config.row_gap = gap;
        self.config.column_gap = gap;
        self
    }

    pub fn with_padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    pub fn add_child(&mut self, child: Box<dyn Widget>) {
        self.children.push(child);
    }
}

impl Widget for Grid {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Grid" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        if self.children.is_empty() || self.config.columns == 0 {
            self.bounds.size = Size::zero();
            return Ok(self.bounds.size);
        }

        let available_width = constraints.max_width - self.padding.horizontal();
        let total_gap = self.config.column_gap * (self.config.columns - 1) as f32;
        let column_width = (available_width - total_gap) / self.config.columns as f32;

        let rows = (self.children.len() + self.config.columns - 1) / self.config.columns;
        
        self.child_rects.clear();
        let mut row_heights = vec![0.0f32; rows];

        // First pass: measure all children and determine row heights
        for (i, child) in self.children.iter_mut().enumerate() {
            let row = i / self.config.columns;
            let child_constraints = LayoutConstraints {
                min_width: 0.0,
                max_width: column_width,
                min_height: 0.0,
                max_height: constraints.max_height,
            };
            let size = child.layout(&child_constraints)?;
            row_heights[row] = row_heights[row].max(size.height);
            self.child_rects.push(Rect::new(0.0, 0.0, size.width, size.height));
        }

        // Calculate total height
        let total_row_gap = self.config.row_gap * (rows - 1) as f32;
        let total_height: f32 = row_heights.iter().sum::<f32>() + total_row_gap;

        self.bounds.size = constraints.constrain(Size::new(
            available_width + self.padding.horizontal(),
            total_height + self.padding.vertical(),
        ));
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;

        if self.children.is_empty() {
            return;
        }

        let content_x = position.x + self.padding.left;
        let content_y = position.y + self.padding.top;
        let available_width = self.bounds.width() - self.padding.horizontal();
        let total_gap = self.config.column_gap * (self.config.columns - 1) as f32;
        let column_width = (available_width - total_gap) / self.config.columns as f32;

        // Calculate row heights
        let rows = (self.children.len() + self.config.columns - 1) / self.config.columns;
        let mut row_heights = vec![0.0f32; rows];
        for (i, rect) in self.child_rects.iter().enumerate() {
            let row = i / self.config.columns;
            row_heights[row] = row_heights[row].max(rect.height());
        }

        // Position children
        let mut y = content_y;
        for row in 0..rows {
            let mut x = content_x;
            let row_height = row_heights[row];

            for col in 0..self.config.columns {
                let idx = row * self.config.columns + col;
                if idx >= self.children.len() {
                    break;
                }

                let child = &mut self.children[idx];
                let child_bounds = child.bounds();
                
                // Center vertically in cell
                let y_offset = (row_height - child_bounds.height()) / 2.0;
                child.set_position(Point::new(x, y + y_offset));

                x += column_width + self.config.column_gap;
            }

            y += row_height + self.config.row_gap;
        }
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, ctx: &UiContext) -> Result<(), UiError> {
        for child in &mut self.children {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &UiContext) -> bool {
        for child in &mut self.children {
            if child.handle_event(event, ctx) {
                return true;
            }
        }
        false
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError> {
        for child in &self.children {
            child.render(ctx)?;
        }
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn children(&self) -> &[Box<dyn Widget>] { &self.children }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { &mut self.children }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Positioned Layout
// =============================================================================

/// Absolute positioning wrapper
pub struct Positioned {
    id: u64,
    bounds: Rect,
    child: Box<dyn Widget>,
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    width: Option<f32>,
    height: Option<f32>,
    visible: bool,
}

impl Positioned {
    pub fn new(child: Box<dyn Widget>) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            child,
            left: None,
            top: None,
            right: None,
            bottom: None,
            width: None,
            height: None,
            visible: true,
        }
    }

    pub fn left(mut self, value: f32) -> Self {
        self.left = Some(value);
        self
    }

    pub fn top(mut self, value: f32) -> Self {
        self.top = Some(value);
        self
    }

    pub fn right(mut self, value: f32) -> Self {
        self.right = Some(value);
        self
    }

    pub fn bottom(mut self, value: f32) -> Self {
        self.bottom = Some(value);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

impl Widget for Positioned {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Positioned" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        // Calculate child constraints based on position/size
        let child_width = self.width.unwrap_or(
            match (self.left, self.right) {
                (Some(l), Some(r)) => constraints.max_width - l - r,
                _ => constraints.max_width,
            }
        );
        let child_height = self.height.unwrap_or(
            match (self.top, self.bottom) {
                (Some(t), Some(b)) => constraints.max_height - t - b,
                _ => constraints.max_height,
            }
        );

        let child_constraints = LayoutConstraints {
            min_width: 0.0,
            max_width: child_width,
            min_height: 0.0,
            max_height: child_height,
        };

        let size = self.child.layout(&child_constraints)?;
        self.bounds.size = size;
        Ok(size)
    }

    fn set_position(&mut self, _position: Point) {
        // Position is determined by left/top/right/bottom relative to parent
        // This is handled by the parent container
        let x = self.left.unwrap_or(0.0);
        let y = self.top.unwrap_or(0.0);
        self.bounds.origin = Point::new(x, y);
        self.child.set_position(self.bounds.origin);
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, ctx: &UiContext) -> Result<(), UiError> {
        self.child.update(ctx)
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &UiContext) -> bool {
        self.child.handle_event(event, ctx)
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError> {
        self.child.render(ctx)
    }

    fn is_visible(&self) -> bool { self.visible }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Spacer
// =============================================================================

/// Flexible spacer for flex layouts
pub struct Spacer {
    id: u64,
    bounds: Rect,
    flex: f32,
}

impl Spacer {
    pub fn new() -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            flex: 1.0,
        }
    }

    pub fn with_flex(mut self, flex: f32) -> Self {
        self.flex = flex;
        self
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spacer {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Spacer" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        // Spacer expands to fill available space
        self.bounds.size = Size::new(constraints.max_width, constraints.max_height);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
    }

    fn bounds(&self) -> Rect { self.bounds }
    fn update(&mut self, _ctx: &UiContext) -> Result<(), UiError> { Ok(()) }
    fn handle_event(&mut self, _event: &UiEvent, _ctx: &UiContext) -> bool { false }
    fn render(&self, _ctx: &mut RenderContext) -> Result<(), UiError> { Ok(()) }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// SizedBox
// =============================================================================

/// Fixed size container
pub struct SizedBox {
    id: u64,
    bounds: Rect,
    child: Option<Box<dyn Widget>>,
    width: Option<f32>,
    height: Option<f32>,
    visible: bool,
}

impl SizedBox {
    pub fn new() -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            child: None,
            width: None,
            height: None,
            visible: true,
        }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_child(mut self, child: Box<dyn Widget>) -> Self {
        self.child = Some(child);
        self
    }

    pub fn shrink(child: Box<dyn Widget>) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            child: Some(child),
            width: None,
            height: None,
            visible: true,
        }
    }
}

impl Default for SizedBox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for SizedBox {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "SizedBox" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let child_constraints = LayoutConstraints {
            min_width: self.width.unwrap_or(constraints.min_width),
            max_width: self.width.unwrap_or(constraints.max_width),
            min_height: self.height.unwrap_or(constraints.min_height),
            max_height: self.height.unwrap_or(constraints.max_height),
        };

        if let Some(child) = &mut self.child {
            let _ = child.layout(&child_constraints)?;
        }

        let size = Size::new(
            self.width.unwrap_or(0.0),
            self.height.unwrap_or(0.0),
        );
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
        if let Some(child) = &mut self.child {
            child.set_position(position);
        }
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, ctx: &UiContext) -> Result<(), UiError> {
        if let Some(child) = &mut self.child {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &UiContext) -> bool {
        if let Some(child) = &mut self.child {
            return child.handle_event(event, ctx);
        }
        false
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError> {
        if let Some(child) = &self.child {
            child.render(ctx)?;
        }
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Aspect Ratio
// =============================================================================

/// Maintains aspect ratio
pub struct AspectRatio {
    id: u64,
    bounds: Rect,
    child: Option<Box<dyn Widget>>,
    ratio: f32,
    visible: bool,
}

impl AspectRatio {
    pub fn new(ratio: f32) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            child: None,
            ratio,
            visible: true,
        }
    }

    pub fn with_child(mut self, child: Box<dyn Widget>) -> Self {
        self.child = Some(child);
        self
    }
}

impl Widget for AspectRatio {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "AspectRatio" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let width = constraints.max_width;
        let height = width / self.ratio;

        let (final_width, final_height) = if height <= constraints.max_height {
            (width, height)
        } else {
            let h = constraints.max_height;
            (h * self.ratio, h)
        };

        let child_constraints = LayoutConstraints::tight(Size::new(final_width, final_height));
        if let Some(child) = &mut self.child {
            let _ = child.layout(&child_constraints)?;
        }

        self.bounds.size = Size::new(final_width, final_height);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
        if let Some(child) = &mut self.child {
            child.set_position(position);
        }
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, ctx: &UiContext) -> Result<(), UiError> {
        if let Some(child) = &mut self.child {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &UiContext) -> bool {
        if let Some(child) = &mut self.child {
            return child.handle_event(event, ctx);
        }
        false
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError> {
        if let Some(child) = &self.child {
            child.render(ctx)?;
        }
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flex_row() {
        let flex = Flex::row();
        assert_eq!(flex.type_name(), "Flex");
    }

    #[test]
    fn test_flex_column() {
        let flex = Flex::column();
        assert_eq!(flex.type_name(), "Flex");
    }

    #[test]
    fn test_stack_alignment() {
        let alignment = Alignment::Center;
        let offset = alignment.offset_for_size(
            Size::new(100.0, 100.0),
            Size::new(50.0, 50.0),
        );
        assert!((offset.x - 25.0).abs() < 0.001);
        assert!((offset.y - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_grid_layout() {
        let grid = Grid::new(3);
        assert_eq!(grid.config.columns, 3);
    }

    #[test]
    fn test_sized_box() {
        let sized = SizedBox::new().with_size(100.0, 50.0);
        assert_eq!(sized.width, Some(100.0));
        assert_eq!(sized.height, Some(50.0));
    }

    #[test]
    fn test_aspect_ratio() {
        let aspect = AspectRatio::new(16.0 / 9.0);
        assert!((aspect.ratio - 1.777).abs() < 0.01);
    }

    #[test]
    fn test_spacer() {
        let spacer = Spacer::new().with_flex(2.0);
        assert!((spacer.flex - 2.0).abs() < 0.001);
    }
}
