use crate::ast::{ChartKind, ChartNode, InlineKind, InlineNode, RowNode};

const PALETTE: [&str; 6] = [
    "#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4",
];

#[derive(Debug, Clone)]
pub struct Series {
    pub name: String,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub series: Vec<Series>,
}

/// Pull labels and numeric series out of a chart or graph.
/// The error string is a semantic message. The line and column belong to the row or cell.
pub fn chart_data(chart: &ChartNode) -> Result<ChartData, (usize, usize, String)> {
    if chart.rows.is_empty() {
        return Err((
            chart.line,
            chart.col,
            "chart must contain at least one row".to_string(),
        ));
    }

    let mut header: Option<&RowNode> = None;
    let mut data_rows = Vec::new();
    for row in &chart.rows {
        if row.header {
            if header.is_some() {
                return Err((
                    row.line,
                    row.col,
                    "chart has more than one header row".to_string(),
                ));
            }
            header = Some(row);
        } else {
            data_rows.push(row);
        }
    }
    if data_rows.is_empty() {
        return Err((
            chart.line,
            chart.col,
            "chart needs at least one data row".to_string(),
        ));
    }

    let width = data_rows[0].cells.len();
    if width < 2 {
        return Err((
            data_rows[0].line,
            data_rows[0].col,
            "chart row needs a label cell and at least one value cell".to_string(),
        ));
    }
    for row in chart.rows.iter() {
        if row.cells.is_empty() {
            return Err((
                row.line,
                row.col,
                "row must contain at least one cell".to_string(),
            ));
        }
        if row.cells.len() != width {
            return Err((
                row.line,
                row.col,
                format!(
                    "row has {} cells, expected {width} to match the other rows",
                    row.cells.len()
                ),
            ));
        }
    }

    let mut series_names = Vec::new();
    if let Some(head) = header {
        for cell in head.cells.iter().skip(1) {
            let name = cell_text(cell);
            series_names.push(if name.is_empty() {
                format!("Series {}", series_names.len() + 1)
            } else {
                name
            });
        }
    } else {
        for i in 1..width {
            series_names.push(format!("Series {i}"));
        }
    }

    let mut labels = Vec::new();
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); width - 1];
    for row in &data_rows {
        let label = cell_text(&row.cells[0]);
        labels.push(if label.is_empty() {
            format!("{}", labels.len() + 1)
        } else {
            label
        });
        for (i, cell) in row.cells.iter().skip(1).enumerate() {
            let text = cell_text(cell);
            let value = parse_number(&text).ok_or_else(|| {
                (
                    cell.line,
                    cell.col,
                    format!("chart value `{text}` is not a number"),
                )
            })?;
            if chart.kind == ChartKind::Pie && value < 0.0 {
                return Err((
                    cell.line,
                    cell.col,
                    "pie charts cannot use a negative value".to_string(),
                ));
            }
            columns[i].push(value);
        }
    }

    if chart.kind == ChartKind::Pie && columns[0].iter().all(|v| *v == 0.0) {
        return Err((
            chart.line,
            chart.col,
            "pie chart values are all zero".to_string(),
        ));
    }

    let series = series_names
        .into_iter()
        .zip(columns)
        .map(|(name, values)| Series { name, values })
        .collect();
    Ok(ChartData { labels, series })
}

pub fn render_chart_svg(chart: &ChartNode) -> String {
    let data = match chart_data(chart) {
        Ok(data) => data,
        Err((_, _, msg)) => {
            return format!(
                "<svg class=\"chart\" viewBox=\"0 0 680 80\" role=\"img\"><text x=\"16\" y=\"44\">{msg}</text></svg>"
            );
        }
    };
    match chart.kind {
        ChartKind::Pie => render_pie(chart, &data),
        ChartKind::Bar => render_bars(chart, &data),
        ChartKind::Line => render_lines(chart, &data),
    }
}

fn render_bars(chart: &ChartNode, data: &ChartData) -> String {
    let (min_v, max_v) = padded_range(data, true);
    let mut out = svg_open(chart);
    draw_title(&mut out, &chart.title);
    draw_y_axis(&mut out, min_v, max_v);
    draw_baseline(&mut out, min_v, max_v);

    let n = data.labels.len().max(1) as f64;
    let groups = data.series.len().max(1) as f64;
    let slot = plot_w() / n;
    let bar_w = (slot * 0.72 / groups).max(2.0);

    for (si, series) in data.series.iter().enumerate() {
        let color = PALETTE[si % PALETTE.len()];
        for (i, value) in series.values.iter().enumerate() {
            let x0 = plot_x() + slot * i as f64 + slot * 0.14 + bar_w * si as f64;
            let y = y_at(*value, min_v, max_v);
            let y0 = y_at(0.0_f64.clamp(min_v, max_v), min_v, max_v);
            let top = y.min(y0);
            let h = (y - y0).abs().max(1.0);
            out.push_str(&format!(
                "<rect x=\"{x0:.1}\" y=\"{top:.1}\" width=\"{bar_w:.1}\" height=\"{h:.1}\" fill=\"{color}\" rx=\"2\"><title>{}: {value}</title></rect>",
                xml_escape(&series.name)
            ));
        }
    }
    draw_x_labels(&mut out, &data.labels);
    draw_legend(&mut out, data);
    out.push_str("</svg>");
    out
}

fn render_lines(chart: &ChartNode, data: &ChartData) -> String {
    let (min_v, max_v) = padded_range(data, false);
    let mut out = svg_open(chart);
    draw_title(&mut out, &chart.title);
    draw_y_axis(&mut out, min_v, max_v);
    draw_baseline(&mut out, min_v, max_v);

    let n = data.labels.len().max(1);
    let span = (n.saturating_sub(1).max(1)) as f64;
    for (si, series) in data.series.iter().enumerate() {
        let color = PALETTE[si % PALETTE.len()];
        let mut points = String::new();
        for (i, value) in series.values.iter().enumerate() {
            let x = if n == 1 {
                plot_x() + plot_w() / 2.0
            } else {
                plot_x() + plot_w() * (i as f64) / span
            };
            let y = y_at(*value, min_v, max_v);
            points.push_str(&format!("{x:.1},{y:.1} "));
        }
        out.push_str(&format!(
            "<polyline fill=\"none\" stroke=\"{color}\" stroke-width=\"2.5\" points=\"{points}\" />"
        ));
        for (i, value) in series.values.iter().enumerate() {
            let x = if n == 1 {
                plot_x() + plot_w() / 2.0
            } else {
                plot_x() + plot_w() * (i as f64) / span
            };
            let y = y_at(*value, min_v, max_v);
            out.push_str(&format!(
                "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"3.5\" fill=\"{color}\"><title>{}: {value}</title></circle>",
                xml_escape(&series.name)
            ));
        }
    }
    draw_x_labels(&mut out, &data.labels);
    draw_legend(&mut out, data);
    out.push_str("</svg>");
    out
}

fn render_pie(chart: &ChartNode, data: &ChartData) -> String {
    let series = &data.series[0];
    let total: f64 = series.values.iter().sum();
    let mut out = svg_open(chart);
    draw_title(&mut out, &chart.title);
    let cx = 230.0;
    let cy = 200.0;
    let r = 110.0;
    if series.values.len() == 1 || total <= 0.0 {
        let color = PALETTE[0];
        out.push_str(&format!(
            "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"{color}\" />"
        ));
    } else {
        let mut angle = -std::f64::consts::FRAC_PI_2;
        for (i, value) in series.values.iter().enumerate() {
            let sweep = (*value / total) * std::f64::consts::TAU;
            let next = angle + sweep;
            let color = PALETTE[i % PALETTE.len()];
            let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
            let x1 = cx + r * angle.cos();
            let y1 = cy + r * angle.sin();
            let x2 = cx + r * next.cos();
            let y2 = cy + r * next.sin();
            out.push_str(&format!(
                "<path d=\"M {cx:.2} {cy:.2} L {x1:.2} {y1:.2} A {r} {r} 0 {large} 1 {x2:.2} {y2:.2} Z\" fill=\"{color}\"><title>{}: {value}</title></path>",
                xml_escape(&data.labels.get(i).map(|s| s.as_str()).unwrap_or(""))
            ));
            angle = next;
        }
    }
    let mut legend = ChartData {
        labels: data.labels.clone(),
        series: data
            .labels
            .iter()
            .enumerate()
            .map(|(i, name)| Series {
                name: name.clone(),
                values: vec![series.values.get(i).copied().unwrap_or(0.0)],
            })
            .collect(),
    };
    // Reuse the legend drawer by naming slices as series.
    legend.series = data
        .labels
        .iter()
        .map(|name| Series {
            name: name.clone(),
            values: Vec::new(),
        })
        .collect();
    draw_legend(&mut out, &legend);
    out.push_str("</svg>");
    out
}

fn svg_open(chart: &ChartNode) -> String {
    let label = if chart.title.is_empty() {
        chart.kind.as_str().to_string()
    } else {
        chart.title.clone()
    };
    format!(
        "<svg class=\"chart\" viewBox=\"0 0 680 380\" role=\"img\" aria-label=\"{}\">",
        xml_escape(&label)
    )
}

fn draw_title(out: &mut String, title: &str) {
    if title.is_empty() {
        return;
    }
    out.push_str(&format!(
        "<text x=\"340\" y=\"24\" text-anchor=\"middle\" class=\"chart-title\">{}</text>",
        xml_escape(title)
    ));
}

fn draw_y_axis(out: &mut String, min_v: f64, max_v: f64) {
    for i in 0..5 {
        let t = i as f64 / 4.0;
        let value = min_v + (max_v - min_v) * (1.0 - t);
        let y = plot_top() + plot_h() * t;
        out.push_str(&format!(
            "<line x1=\"48\" y1=\"{y:.1}\" x2=\"620\" y2=\"{y:.1}\" class=\"chart-grid\" />"
        ));
        out.push_str(&format!(
            "<text x=\"42\" y=\"{:.1}\" text-anchor=\"end\" class=\"chart-label\">{}</text>",
            y + 4.0,
            fmt_num(value)
        ));
    }
}

fn draw_baseline(out: &mut String, min_v: f64, max_v: f64) {
    if min_v < 0.0 && max_v > 0.0 {
        let y = y_at(0.0, min_v, max_v);
        out.push_str(&format!(
            "<line x1=\"48\" y1=\"{y:.1}\" x2=\"620\" y2=\"{y:.1}\" class=\"chart-axis\" />"
        ));
    }
}

fn draw_x_labels(out: &mut String, labels: &[String]) {
    let n = labels.len().max(1) as f64;
    let slot = plot_w() / n;
    for (i, label) in labels.iter().enumerate() {
        let x = plot_x() + slot * i as f64 + slot / 2.0;
        out.push_str(&format!(
            "<text x=\"{x:.1}\" y=\"336\" text-anchor=\"middle\" class=\"chart-label\">{}</text>",
            xml_escape(&shorten(label, 16))
        ));
    }
}

fn draw_legend(out: &mut String, data: &ChartData) {
    let mut x = 48.0;
    for (i, series) in data.series.iter().enumerate() {
        let color = PALETTE[i % PALETTE.len()];
        out.push_str(&format!(
            "<rect x=\"{x:.1}\" y=\"352\" width=\"12\" height=\"12\" fill=\"{color}\" rx=\"2\" />"
        ));
        x += 16.0;
        out.push_str(&format!(
            "<text x=\"{x:.1}\" y=\"362\" class=\"chart-label\">{}</text>",
            xml_escape(&shorten(&series.name, 18))
        ));
        x += 18.0 + (shorten(&series.name, 18).chars().count() as f64) * 7.0;
        if x > 620.0 {
            break;
        }
    }
}

fn padded_range(data: &ChartData, bars: bool) -> (f64, f64) {
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for series in &data.series {
        for v in &series.values {
            min_v = min_v.min(*v);
            max_v = max_v.max(*v);
        }
    }
    if !min_v.is_finite() || !max_v.is_finite() {
        return (0.0, 1.0);
    }
    if bars && min_v > 0.0 {
        min_v = 0.0;
    }
    if min_v == max_v {
        if min_v == 0.0 {
            return (0.0, 1.0);
        }
        let pad = min_v.abs() * 0.2;
        return (min_v - pad, max_v + pad);
    }
    let pad = (max_v - min_v) * 0.1;
    let mut lo = min_v - pad;
    let hi = max_v + pad;
    if min_v >= 0.0 && lo < 0.0 {
        lo = 0.0;
    }
    (lo, hi)
}

fn y_at(value: f64, min_v: f64, max_v: f64) -> f64 {
    let span = (max_v - min_v).max(f64::EPSILON);
    plot_top() + plot_h() * (1.0 - (value - min_v) / span)
}

fn plot_x() -> f64 {
    56.0
}
fn plot_top() -> f64 {
    40.0
}
fn plot_w() -> f64 {
    560.0
}
fn plot_h() -> f64 {
    270.0
}

fn fmt_num(n: f64) -> String {
    if n.abs() >= 100.0 || n.fract().abs() < 0.05 {
        format!("{n:.0}")
    } else {
        format!("{n:.1}")
    }
}

fn shorten(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn cell_text(cell: &crate::ast::CellNode) -> String {
    plain(&cell.children)
}

fn plain(inlines: &[InlineNode]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match &inline.kind {
            InlineKind::Text(s) | InlineKind::Code(s) | InlineKind::Math(s) => out.push_str(s),
            InlineKind::Bold(c) | InlineKind::Italic(c) | InlineKind::Strike(c) => {
                out.push_str(&plain(c))
            }
            InlineKind::Link { children, .. } => out.push_str(&plain(children)),
            InlineKind::FootnoteRef(_) => {}
        }
    }
    out.trim().to_string()
}

fn parse_number(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    t.parse::<f64>().ok().filter(|n| n.is_finite())
}

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => push_entity(&mut out, "amp"),
            '<' => push_entity(&mut out, "lt"),
            '>' => push_entity(&mut out, "gt"),
            '"' => push_entity(&mut out, "quot"),
            _ => out.push(c),
        }
    }
    out
}

fn push_entity(out: &mut String, name: &str) {
    out.push('&');
    out.push_str(name);
    out.push(';');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{CellNode, ChartKind, ChartNode, RowNode};

    fn cell(text: &str) -> CellNode {
        CellNode {
            line: 1,
            col: 1,
            children: vec![InlineNode::text(text)],
        }
    }

    fn row(header: bool, values: &[&str]) -> RowNode {
        RowNode {
            line: 1,
            col: 1,
            header,
            cells: values.iter().map(|v| cell(v)).collect(),
        }
    }

    #[test]
    fn bar_svg_has_rects() {
        let chart = ChartNode {
            line: 1,
            col: 1,
            kind: ChartKind::Bar,
            title: "Passes".into(),
            as_graph: false,
            rows: vec![
                row(true, &["Step", "Count"]),
                row(false, &["parse", "3"]),
                row(false, &["fmt", "9"]),
            ],
        };
        let svg = render_chart_svg(&chart);
        assert!(svg.contains("rect "), "{svg}");
        assert!(svg.contains("Passes"), "{svg}");
    }

    #[test]
    fn rejects_bad_number() {
        let chart = ChartNode {
            line: 2,
            col: 3,
            kind: ChartKind::Bar,
            title: String::new(),
            as_graph: false,
            rows: vec![row(false, &["a", "no"])],
        };
        let err = chart_data(&chart).unwrap_err();
        assert!(err.2.len() > 0);
    }
}
