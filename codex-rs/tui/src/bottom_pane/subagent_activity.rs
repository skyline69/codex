//! A quiet, single-row summary of delegated work, independent of the parent's turn status.

use crate::render::renderable::Renderable;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use std::time::Duration;
use std::time::Instant;

const COMPLETION_DISPLAY_DURATION: Duration = Duration::from_secs(/*secs*/ 5);

/// Retains completion briefly without extending its deadline on routine state synchronization.
#[derive(Default)]
pub(super) struct SubagentActivityIndicator {
    activity: SubagentActivity,
    pub(super) hide_at: Option<Instant>,
}

impl SubagentActivityIndicator {
    pub(super) fn update(&mut self, activity: SubagentActivity, now: Instant) -> bool {
        if self.activity == activity {
            return false;
        }
        self.activity = activity;
        self.hide_at = (activity.running == 0 && activity.idle == 0 && activity.done > 0)
            .then_some(now + COMPLETION_DISPLAY_DURATION);
        true
    }

    pub(super) fn visible_activity(&self, now: Instant) -> Option<&SubagentActivity> {
        if self.activity.is_empty() || self.hide_at.is_some_and(|deadline| now >= deadline) {
            None
        } else {
            Some(&self.activity)
        }
    }
}

/// Session-wide counts. Idle threads have no confirmed running or terminal state yet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SubagentActivity {
    pub(crate) running: usize,
    pub(crate) done: usize,
    pub(crate) idle: usize,
}

impl SubagentActivity {
    pub(crate) fn is_empty(&self) -> bool {
        self.running + self.done + self.idle == 0
    }

    fn line(&self, width: u16) -> Line<'static> {
        let marker = if self.running > 0 {
            "● ".cyan()
        } else {
            "○ ".dim()
        };
        let mut spans = vec!["  ".into(), marker, "Agents  ".dim()];
        let mut counts = Vec::new();
        if self.running > 0 {
            counts.push(format!("{} running", self.running).cyan());
        }
        if self.done > 0 {
            counts.push(format!("{} done", self.done).dim());
        }
        if self.idle > 0 {
            counts.push(format!("{} idle", self.idle).dim());
        }
        for (index, count) in counts.into_iter().enumerate() {
            if index > 0 {
                spans.push(" · ".dim());
            }
            spans.push(count);
        }
        let mut line = Line::from(spans);
        if line.width() > usize::from(width) {
            let running = self.running;
            let total = self.running + self.done + self.idle;
            line = vec![
                if running > 0 {
                    "● ".cyan()
                } else {
                    "○ ".dim()
                },
                format!("{running}/{total}").dim(),
            ]
            .into();
            if line.width() + 7 <= usize::from(width) {
                line.spans.push(" active".dim());
            }
        }
        if line.width() + 13 <= usize::from(width) {
            line.spans.push("  /subagents".dim());
        }
        line
    }
}

impl Renderable for SubagentActivity {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.is_empty() && !area.is_empty() {
            Paragraph::new(self.line(area.width)).render(area, buf);
        }
    }

    fn desired_height(&self, width: u16) -> u16 {
        u16::from(!self.is_empty() && width > 0)
    }
}

#[cfg(test)]
#[path = "subagent_activity_tests.rs"]
mod tests;
