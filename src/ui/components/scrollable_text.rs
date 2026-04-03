//! Scrollable text viewport with scrollbar.
//!
//! Renders a FIXED number of line rows (`viewport_rows`) regardless of content length.
//! Content is windowed by `scroll_offset`. Empty rows are padded so the component
//! always occupies the same layout space — preventing layout shifts.

use iocraft::prelude::*;

/// Props for the scrollable text viewport.
#[derive(Default, Props)]
pub struct ScrollableTextProps {
    /// The full text content to display (may contain newlines).
    pub content: String,
    /// Current scroll offset in lines (0 = top).
    pub scroll_offset: usize,
    /// Fixed number of rows this viewport occupies. Must be set by the parent.
    /// The component always renders exactly this many `Box(height: 1u32)` elements.
    pub viewport_rows: usize,
    /// Text color.
    pub color: Option<Color>,
    /// Scrollbar track color (dimmed).
    pub track_color: Option<Color>,
    /// Scrollbar thumb color (bright).
    pub thumb_color: Option<Color>,
}

/// Compute scrollbar thumb position and size using integer math.
fn scrollbar_geometry(total: usize, visible: usize, offset: usize) -> (usize, usize) {
    if total <= visible || visible == 0 {
        return (0, visible);
    }
    let thumb_size = (visible * visible / total).max(1).min(visible);
    let max_offset = total.saturating_sub(visible);
    let scrollable_rows = visible.saturating_sub(thumb_size);
    let thumb_pos = if max_offset > 0 {
        (offset * scrollable_rows / max_offset).min(scrollable_rows)
    } else {
        0
    };
    (thumb_pos, thumb_size)
}

/// Scrollable text viewport — renders exactly `viewport_rows` line boxes.
///
/// Content is windowed from `scroll_offset`. Lines beyond content are blank.
/// A scrollbar is drawn on the right when content exceeds the viewport.
#[component]
pub fn ScrollableText(props: &ScrollableTextProps) -> impl Into<AnyElement<'static>> {
    let fg = props.color.unwrap_or(Color::Reset);
    let track_color = props.track_color.unwrap_or(Color::DarkGrey);
    let thumb_color = props.thumb_color.unwrap_or(Color::White);
    let vp = props.viewport_rows.max(1);

    let all_lines: Vec<&str> = if props.content.is_empty() {
        Vec::new()
    } else {
        props.content.lines().collect()
    };
    let total = all_lines.len();
    let max_offset = total.saturating_sub(vp);
    let offset = props.scroll_offset.min(max_offset);

    // Build exactly `vp` display lines — pad with empty if content is short
    let display_lines: Vec<String> = (0..vp)
        .map(|row| {
            let line_idx = offset + row;
            if line_idx < total {
                all_lines[line_idx].to_string()
            } else {
                String::new()
            }
        })
        .collect();

    let show_scrollbar = total > vp;
    let (thumb_pos, thumb_size) = scrollbar_geometry(total, vp, offset);

    element! {
        Box(flex_direction: FlexDirection::Row, width: 100pct) {
            // Text content column — exactly `vp` rows
            Box(flex_direction: FlexDirection::Column, flex_grow: 1.0) {
                #(display_lines.iter().map(|line| {
                    element! {
                        Box(height: 1u32) {
                            Text(content: line.clone(), color: fg, wrap: TextWrap::NoWrap)
                        }
                    }
                }).collect::<Vec<_>>())
            }
            // Scrollbar column (1 char wide, same `vp` rows)
            #(if show_scrollbar {
                vec![element! {
                    Box(flex_direction: FlexDirection::Column, width: 1u32) {
                        #((0..vp).map(|row| {
                            let is_thumb = row >= thumb_pos && row < thumb_pos + thumb_size;
                            let ch = if is_thumb { "┃" } else { "│" };
                            let color = if is_thumb { thumb_color } else { track_color };
                            element! {
                                Box(height: 1u32) {
                                    Text(content: ch.to_string(), color: color)
                                }
                            }
                        }).collect::<Vec<_>>())
                    }
                }]
            } else {
                vec![]
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollbar_geometry_no_scroll() {
        let (pos, size) = scrollbar_geometry(5, 10, 0);
        assert_eq!(pos, 0);
        assert_eq!(size, 10);
    }

    #[test]
    fn test_scrollbar_geometry_at_top() {
        let (pos, size) = scrollbar_geometry(100, 20, 0);
        assert_eq!(pos, 0);
        assert!(size >= 1);
        assert!(size <= 20);
    }

    #[test]
    fn test_scrollbar_geometry_at_bottom() {
        let (pos, size) = scrollbar_geometry(100, 20, 80);
        assert!(pos + size <= 20);
    }
}
