use super::{block_style, default_style, highlight_style, warning_style, BLOCK_BORDERS};
use crate::data::{bytes_to_gb, bytes_to_mb, DiskData};
use crate::i18n;
use ratatui::{
    layout::Rect,
    widgets::{Block, Row, Table},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, data: &DiskData) {
    #[cfg(target_os = "windows")]
    let disk_rows: Vec<Row> = data
        .disks
        .iter()
        .map(|(mount, used, total, _read_speed, _write_speed)| {
            let used_gb = bytes_to_gb(*used);
            let total_gb = bytes_to_gb(*total);
            let percent = if total_gb > 0.0 {
                (used_gb / total_gb) * 100.0
            } else {
                0.0
            };

            Row::new(vec![
                mount.clone(),
                format!("{:.1}GB", used_gb),
                format!("{:.1}GB", total_gb),
                format!("{:.1}%", percent),
            ])
            .style(if percent > 80.0 {
                warning_style()
            } else {
                default_style()
            })
        })
        .collect();

    #[cfg(not(target_os = "windows"))]
    let disk_rows: Vec<Row> = data
        .disks
        .iter()
        .map(|(mount, used, total, read_speed, write_speed)| {
            let used_gb = bytes_to_gb(*used);
            let total_gb = bytes_to_gb(*total);
            let percent = if total_gb > 0.0 {
                (used_gb / total_gb) * 100.0
            } else {
                0.0
            };

            let read_mb = bytes_to_mb(*read_speed);
            let write_mb = bytes_to_mb(*write_speed);

            Row::new(vec![
                mount.clone(),
                format!("{:.1}GB", used_gb),
                format!("{:.1}GB", total_gb),
                format!("{:.1}%", percent),
                format!("{:.1}MB/s", read_mb),
                format!("{:.1}MB/s", write_mb),
            ])
            .style(if percent > 80.0 {
                warning_style()
            } else {
                default_style()
            })
        })
        .collect();

    #[cfg(target_os = "windows")]
    let disk_table = Table::new(
        disk_rows,
        [
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(25),
        ],
    );

    #[cfg(not(target_os = "windows"))]
    let disk_table = Table::new(
        disk_rows,
        [
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(15),
            ratatui::layout::Constraint::Percentage(15),
            ratatui::layout::Constraint::Percentage(15),
            ratatui::layout::Constraint::Percentage(17),
            ratatui::layout::Constraint::Percentage(18),
        ],
    );

    #[cfg(target_os = "windows")]
    let header = Row::new(vec![
        i18n::t("mount_point"),
        i18n::t("used"),
        i18n::t("total"),
        i18n::t("usage"),
    ]);

    #[cfg(not(target_os = "windows"))]
    let header = Row::new(vec![
        i18n::t("mount_point"),
        i18n::t("used"),
        i18n::t("total"),
        i18n::t("usage"),
        i18n::t("read_speed"),
        i18n::t("write_speed"),
    ]);

    let disk_table = disk_table
        .block(
            Block::default()
                .title(i18n::t("disk_usage"))
                .borders(BLOCK_BORDERS)
                .style(block_style()),
        )
        .header(header.style(highlight_style()));

    f.render_widget(disk_table, area);
}