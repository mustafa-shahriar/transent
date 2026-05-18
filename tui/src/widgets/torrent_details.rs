use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Padding, Paragraph, Row, Table, Wrap};
use transmission_rpc::types::{Torrent, TorrentStatus};

use crate::config::Theme;
use crate::util::{readabl_eta, readable_size, readable_time, readble_speed, status_to_string};

pub struct Details {
    pub torrent: Option<Torrent>,
}

impl Details {
    pub fn new() -> Self {
        Self { torrent: None }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let bg = Theme::color(&theme.general.background);
        let fg = Theme::color(&theme.general.foreground);
        let muted = Theme::color(&theme.details.muted_fg);
        let accent = Theme::color(&theme.details.accent_fg);
        let card_bg = Theme::color(&theme.details.card_bg);
        let border_c = Theme::color(&theme.details.border_color);
        let filled = Theme::color(&theme.progress_bar.filled);
        let empty_c = Theme::color(&theme.progress_bar.empty);

        let base_style = Style::default().fg(fg).bg(bg);
        let muted_style = Style::default().fg(muted).bg(bg);
        let border_style = Style::default().fg(border_c).bg(bg);

        let outer_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" Torrent Details ", border_style))
            .border_style(border_style)
            .padding(Padding::new(1, 1, 1, 1));

        let Some(torrent) = &self.torrent else {
            let p =
                Paragraph::new(Span::styled("No torrent selected", muted_style)).block(outer_block);
            frame.render_widget(p, area);
            return;
        };

        let name = torrent.name.clone().unwrap_or_default();
        let total_size_bytes = torrent.total_size.unwrap_or(0) as u64;
        let downloaded_bytes = torrent.downloaded_ever.unwrap_or(0);
        let total_size = readable_size(total_size_bytes);
        let downloaded = readable_size(downloaded_bytes);
        let uploaded = readable_size(torrent.uploaded_ever.unwrap_or(0) as u64);
        let status = status_to_string(torrent.status.unwrap());
        let progress = torrent.percent_done.unwrap_or(0.0).clamp(0.0, 1.0) as f64;
        let down_speed = readble_speed(torrent.rate_download.unwrap_or(0));
        let up_speed = readble_speed(torrent.rate_upload.unwrap_or(0));
        let eta = readabl_eta(torrent.eta.unwrap_or(-1));
        let peers = torrent.peers_connected.unwrap_or(0).to_string();
        let seed_time = readable_time(torrent.seconds_seeding.unwrap_or(0));
        let remaining = readable_size(total_size_bytes.saturating_sub(downloaded_bytes));

        // status dot color: accent while downloading, success when seeding
        let status_str = torrent.status.unwrap();
        let dot_color = if status_str == TorrentStatus::Seeding {
            // seeding
            Theme::color(&theme.details.success_fg)
        } else {
            accent
        };

        let inner = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        // ── Vertical sections ─────────────────────────────────────────────
        //  0  filename          2 lines
        //  1  status row        1 line
        //  2  spacer            1 line
        //  3  progress bar      1 line
        //  4  bytes label       1 line
        //  5  spacer            1 line
        //  6  stat cards        3 lines
        //  7  spacer            1 line
        //  8  detail rows       remaining
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(4),
                Constraint::Length(1),
                Constraint::Min(2),
            ])
            .split(inner);

        // ── [0] Filename ──────────────────────────────────────────────────
        frame.render_widget(
            Paragraph::new(name.as_str())
                .style(Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD))
                .wrap(Wrap { trim: true }),
            sections[0],
        );

        // ── [1] Status dot + peers ─────────────────────────────────────────
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(dot_color).bg(bg)),
                Span::styled(
                    format!("{status}  "),
                    Style::default()
                        .fg(dot_color)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{peers} peers connected"), muted_style),
            ])),
            sections[1],
        );

        // ── [3] Progress bar ───────────────────────────────────────────────
        frame.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(filled).bg(empty_c))
                .ratio(progress)
                .label(format!("{:.1}%", progress * 100.0))
                .use_unicode(true),
            sections[3],
        );

        // ── [4] Bytes label ────────────────────────────────────────────────
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    downloaded.clone(),
                    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("  of {total_size} downloaded"), muted_style),
            ])),
            sections[4],
        );

        // ── [6] Stat cards ─────────────────────────────────────────────────
        let card_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(sections[6]);

        let cards = [
            ("↓ Download", down_speed.as_str()),
            ("↑ Upload", up_speed.as_str()),
            ("Uploaded", uploaded.as_str()),
            if torrent.status.unwrap() == TorrentStatus::Downloading {
                ("time remaining", eta.as_str())
            } else {
                ("Seed time", seed_time.as_str())
            },
        ];

        for (i, (label, value)) in cards.iter().enumerate() {
            let card_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_c).bg(card_bg))
                .title(Span::styled(
                    format!(" {label} "),
                    Style::default().fg(muted).bg(card_bg),
                ))
                .style(Style::default().bg(card_bg));

            let card_inner = card_block.inner(card_areas[i]);
            frame.render_widget(card_block, card_areas[i]);
            frame.render_widget(
                Paragraph::new(Span::styled(
                    *value,
                    Style::default()
                        .fg(accent)
                        .bg(card_bg)
                        .add_modifier(Modifier::BOLD),
                )),
                card_inner,
            );
        }

        // ── [8] Two-column detail rows ─────────────────────────────────────
        let detail_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(sections[8]);

        let val_style = Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD);

        let left_rows = vec![
            Row::new(vec![
                Cell::from(Span::styled("Total size", muted_style)),
                Cell::from(Span::styled(total_size.clone(), val_style)),
            ]),
            Row::new(vec![
                Cell::from(Span::styled("Downloaded", muted_style)),
                Cell::from(Span::styled(downloaded.clone(), val_style)),
            ]),
        ];

        let right_rows = vec![
            Row::new(vec![
                Cell::from(Span::styled("Connected peers", muted_style)),
                Cell::from(Span::styled(peers.clone(), val_style)),
            ]),
            Row::new(vec![
                Cell::from(Span::styled("Remaining", muted_style)),
                Cell::from(Span::styled(remaining.clone(), val_style)),
            ]),
        ];

        let col_w = [Constraint::Percentage(55), Constraint::Percentage(45)];

        frame.render_widget(
            Table::new(left_rows, col_w).style(base_style),
            detail_cols[0],
        );
        frame.render_widget(
            Table::new(right_rows, col_w).style(base_style),
            detail_cols[1],
        );
    }
}
