//! Reusable proportional bar visualization module.
//!
//! This module can be used for any data with (label, count) pairs,
//! not just shell history.

use std::io::{self, Write};

use crate::color::{Color, Colorizer};

/// Configuration for bar rendering
pub struct BarConfig {
    /// Width of the bar in characters
    pub size: usize,
    /// Show filled portion (percentage)
    pub show_percentage: bool,
    /// Show semi-filled portion (inverse cumulative)
    pub show_cumulative: bool,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            size: 25,
            show_percentage: true,
            show_cumulative: true,
        }
    }
}

/// A data item to be rendered as a bar
pub struct BarItem<'a> {
    pub label: &'a str,
    pub value: usize,
}

impl<'a> BarItem<'a> {
    #[inline]
    pub fn new(label: &'a str, value: usize) -> Self {
        Self { label, value }
    }
}

/// Rendered bar output for a single item
pub struct RenderedBar {
    pub count_str: String,
    pub bar_str: String,
    pub percentage_str: String,
    pub label: String,
}

/// Render a bar segment given percentage values
#[inline]
fn render_bar_segment(
    perc: f32,
    inv_cumu_perc: f32,
    bar_size: usize,
    show_cumu: bool,
    show_perc: bool,
) -> String {
    let dec = perc / 100.0;
    let inv_cumu_dec = inv_cumu_perc / 100.0;
    let (mut semifilled_count, mut filled_count, mut unfilled_count) = (0, 0, 0);

    match (show_cumu, show_perc) {
        (true, true) => {
            semifilled_count =
                ((inv_cumu_dec - dec) * bar_size as f32).round() as usize;
            filled_count = (dec * bar_size as f32).round() as usize;
            if filled_count + semifilled_count > bar_size {
                semifilled_count = bar_size - filled_count;
            };
            unfilled_count =
                (bar_size - filled_count - semifilled_count).min(bar_size);
        }
        (false, true) => {
            filled_count = (dec * bar_size as f32).round() as usize;
            unfilled_count = (bar_size - filled_count).min(bar_size);
        }
        (true, false) => {
            semifilled_count =
                (inv_cumu_dec * bar_size as f32).round() as usize;
            unfilled_count = (bar_size - semifilled_count).min(bar_size);
        }
        (_, _) => {}
    }

    if unfilled_count + semifilled_count + filled_count > 0 {
        // Pre-allocate: 3 bytes for │ chars + characters for bar
        let mut bar = String::with_capacity(6 + unfilled_count * 3 + semifilled_count * 3 + filled_count * 3);
        bar.push('│');
        for _ in 0..unfilled_count {
            bar.push('░');
        }
        for _ in 0..semifilled_count {
            bar.push('▓');
        }
        for _ in 0..filled_count {
            bar.push('█');
        }
        bar.push('│');
        bar
    } else {
        String::new()
    }
}

/// Render bars for a slice of items.
///
/// Items should be pre-sorted by value (descending).
/// Returns rendered bars with counts, bar graphics, percentages, and labels.
pub fn render_bars<'a>(
    items: &[BarItem<'a>],
    config: &BarConfig,
) -> Vec<RenderedBar> {
    if items.is_empty() {
        return Vec::new();
    }

    let total: usize = items.iter().map(|item| item.value).sum();
    if total == 0 {
        return Vec::new();
    }

    let mut results = Vec::with_capacity(items.len());
    let mut inv_cumu_perc = 100.0;

    // Calculate max widths for alignment
    let max_count_width = items
        .iter()
        .map(|i| i.value.to_string().len())
        .max()
        .unwrap_or(0);

    for item in items {
        let perc = item.value as f32 / total as f32 * 100.0;
        let percentage_str = format!("{:.2}%", perc);

        let bar_str = if config.size > 0 {
            let bar = render_bar_segment(
                perc,
                inv_cumu_perc,
                config.size,
                config.show_cumulative,
                config.show_percentage,
            );
            inv_cumu_perc -= perc;
            bar
        } else {
            String::new()
        };

        let count_str = format!("{:>width$}", item.value, width = max_count_width);

        results.push(RenderedBar {
            count_str,
            bar_str,
            percentage_str,
            label: item.label.to_string(),
        });
    }

    results
}

/// Write rendered bars to a writer with proper alignment and optional colors
pub fn write_bars<W: Write>(
    writer: &mut W,
    bars: &[RenderedBar],
    show_bar: bool,
    colorizer: &Colorizer,
) -> io::Result<()> {
    if bars.is_empty() {
        return Ok(());
    }

    // Calculate max percentage width for alignment
    let max_perc_width = bars
        .iter()
        .map(|b| b.percentage_str.len())
        .max()
        .unwrap_or(0);

    let padding = "   ";

    for bar in bars {
        // Color the count
        let count_display = colorizer.paint(Color::Cyan, &bar.count_str);
        write!(writer, "{}{}", count_display, padding)?;

        if show_bar && !bar.bar_str.is_empty() {
            write!(writer, "{} ", bar.bar_str)?;
        }

        // Color the percentage
        let perc_formatted = format!("{:>width$}", bar.percentage_str, width = max_perc_width);
        let perc_display = colorizer.paint(Color::Yellow, &perc_formatted);

        // Color the label
        let label_display = colorizer.paint(Color::BrightWhite, &bar.label);

        writeln!(writer, "{}{}{}", perc_display, padding, label_display)?;
    }
    Ok(())
}

/// Print rendered bars to stdout with proper alignment and optional colors
/// (convenience wrapper around write_bars)
pub fn print_bars(bars: &[RenderedBar], show_bar: bool, colorizer: &Colorizer) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = write_bars(&mut handle, bars, show_bar, colorizer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorMode;

    #[test]
    fn test_render_bars_empty() {
        let items: Vec<BarItem> = vec![];
        let config = BarConfig::default();
        let result = render_bars(&items, &config);
        assert!(result.is_empty());
    }

    #[test]
    fn test_render_bars_single() {
        let items = vec![BarItem::new("ls", 10)];
        let config = BarConfig::default();
        let result = render_bars(&items, &config);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "ls");
        assert_eq!(result[0].percentage_str, "100.00%");
    }

    #[test]
    fn test_render_bars_multiple() {
        let items = vec![
            BarItem::new("ls", 50),
            BarItem::new("git", 30),
            BarItem::new("cd", 20),
        ];
        let config = BarConfig::default();
        let result = render_bars(&items, &config);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].percentage_str, "50.00%");
    }

    #[test]
    fn test_print_bars_no_crash() {
        let items = vec![BarItem::new("test", 5)];
        let config = BarConfig::default();
        let bars = render_bars(&items, &config);
        let colorizer = Colorizer::new(ColorMode::Never);
        // Just verify it doesn't crash
        print_bars(&bars, true, &colorizer);
    }
}
