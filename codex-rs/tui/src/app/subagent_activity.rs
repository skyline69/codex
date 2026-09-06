//! Mirrors session subagent liveness into the persistent composer summary.

use super::App;

impl App {
    pub(super) fn sync_subagent_activity(&mut self) {
        let activity = self.agent_navigation.activity_summary(|thread_id| {
            self.primary_thread_id.is_some()
                && Some(thread_id) != self.primary_thread_id
                && !self.side_threads.contains_key(&thread_id)
        });
        self.chat_widget.set_subagent_activity(activity);
    }
}
