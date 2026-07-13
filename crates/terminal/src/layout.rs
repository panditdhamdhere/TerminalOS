use serde::{Deserialize, Serialize};
use terminalos_shared::{Error, PaneId, Result};

/// Minimum pane dimensions when splitting.
pub const MIN_PANE_WIDTH: u16 = 20;
pub const MIN_PANE_HEIGHT: u16 = 4;

/// Rectangle for layout math (no ratatui dependency).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Area {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Split orientation within a terminal tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitDirection {
    /// Side-by-side panes.
    Horizontal,
    /// Stacked panes.
    Vertical,
}

/// Tree of terminal panes within a tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SplitNode {
    Leaf {
        pane_id: PaneId,
    },
    Split {
        direction: SplitDirection,
        ratio: f32,
        first: Box<SplitNode>,
        second: Box<SplitNode>,
    },
}

impl SplitNode {
    #[must_use]
    pub fn single(pane_id: PaneId) -> Self {
        Self::Leaf { pane_id }
    }

    #[must_use]
    pub fn pane_count(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Split { first, second, .. } => first.pane_count() + second.pane_count(),
        }
    }

    #[must_use]
    pub fn collect_panes(&self) -> Vec<PaneId> {
        match self {
            Self::Leaf { pane_id } => vec![*pane_id],
            Self::Split { first, second, .. } => {
                let mut panes = first.collect_panes();
                panes.extend(second.collect_panes());
                panes
            }
        }
    }

    pub fn split_pane(
        &mut self,
        pane_id: PaneId,
        direction: SplitDirection,
        new_pane: PaneId,
    ) -> bool {
        match self {
            Self::Leaf { pane_id: existing } if *existing == pane_id => {
                *self = Self::Split {
                    direction,
                    ratio: 0.5,
                    first: Box::new(Self::Leaf { pane_id: *existing }),
                    second: Box::new(Self::Leaf { pane_id: new_pane }),
                };
                true
            }
            Self::Split { first, second, .. } => {
                Self::split_pane(first, pane_id, direction, new_pane)
                    || Self::split_pane(second, pane_id, direction, new_pane)
            }
            _ => false,
        }
    }

    pub fn remove_pane(&mut self, pane_id: PaneId) -> bool {
        match self {
            Self::Leaf { pane_id: existing } => *existing == pane_id,
            Self::Split { first, second, .. } => {
                if let Self::Leaf { pane_id: existing } = first.as_ref() {
                    if *existing == pane_id {
                        *self = second.as_ref().clone();
                        return true;
                    }
                }
                if let Self::Leaf { pane_id: existing } = second.as_ref() {
                    if *existing == pane_id {
                        *self = first.as_ref().clone();
                        return true;
                    }
                }
                Self::remove_pane(first, pane_id) || Self::remove_pane(second, pane_id)
            }
        }
    }

    #[must_use]
    pub fn contains_pane(&self, pane_id: PaneId) -> bool {
        self.collect_panes().contains(&pane_id)
    }
}

/// Serializes a split layout tree to JSON for workspace persistence.
pub fn serialize_layout(node: &SplitNode) -> Result<String> {
    serde_json::to_string(node).map_err(|e| Error::Terminal(format!("serialize layout: {e}")))
}

/// Restores a split layout tree from persisted JSON.
pub fn deserialize_layout(json: &str) -> Result<SplitNode> {
    serde_json::from_str(json).map_err(|e| Error::Terminal(format!("deserialize layout: {e}")))
}

/// Computes render rectangles for each pane in a split layout.
#[must_use]
pub fn compute_pane_rects(area: Area, node: &SplitNode) -> Vec<(PaneId, Area)> {
    match node {
        SplitNode::Leaf { pane_id } => vec![(*pane_id, area)],
        SplitNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let (first_area, second_area) = split_area(area, *direction, *ratio);
            let mut rects = compute_pane_rects(first_area, first);
            rects.extend(compute_pane_rects(second_area, second));
            rects
        }
    }
}

fn split_area(area: Area, direction: SplitDirection, ratio: f32) -> (Area, Area) {
    let ratio = ratio.clamp(0.25, 0.75);
    match direction {
        SplitDirection::Horizontal => {
            let first_width = ((area.width as f32) * ratio) as u16;
            let first_width = first_width
                .max(MIN_PANE_WIDTH)
                .min(area.width.saturating_sub(MIN_PANE_WIDTH));
            let second_width = area.width.saturating_sub(first_width);
            (
                Area {
                    x: area.x,
                    y: area.y,
                    width: first_width,
                    height: area.height,
                },
                Area {
                    x: area.x + first_width,
                    y: area.y,
                    width: second_width,
                    height: area.height,
                },
            )
        }
        SplitDirection::Vertical => {
            let first_height = ((area.height as f32) * ratio) as u16;
            let first_height = first_height
                .max(MIN_PANE_HEIGHT)
                .min(area.height.saturating_sub(MIN_PANE_HEIGHT));
            let second_height = area.height.saturating_sub(first_height);
            (
                Area {
                    x: area.x,
                    y: area.y,
                    width: area.width,
                    height: first_height,
                },
                Area {
                    x: area.x,
                    y: area.y + first_height,
                    width: area.width,
                    height: second_height,
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_leaf_into_two_panes() {
        let left = PaneId::new();
        let right = PaneId::new();
        let mut node = SplitNode::single(left);
        assert!(node.split_pane(left, SplitDirection::Horizontal, right));
        assert_eq!(node.pane_count(), 2);
        assert!(node.contains_pane(left));
        assert!(node.contains_pane(right));
    }

    #[test]
    fn removes_pane_collapses_split() {
        let left = PaneId::new();
        let right = PaneId::new();
        let mut node = SplitNode::single(left);
        node.split_pane(left, SplitDirection::Horizontal, right);
        assert!(node.remove_pane(right));
        assert_eq!(node.pane_count(), 1);
        assert!(node.contains_pane(left));
    }
}
