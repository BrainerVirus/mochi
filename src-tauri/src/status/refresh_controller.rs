use std::collections::HashSet;
use std::sync::Mutex;

use crate::core::models::ProviderId;

#[derive(Default)]
pub struct RefreshController {
    active: Mutex<HashSet<ProviderId>>,
}

pub struct ProviderRefreshGuard<'a> {
    controller: &'a RefreshController,
    provider: ProviderId,
}

impl RefreshController {
    pub fn try_begin_provider_refresh(
        &self,
        provider: ProviderId,
    ) -> Option<ProviderRefreshGuard<'_>> {
        let mut active = self.active.lock().ok()?;
        if !active.insert(provider) {
            return None;
        }
        Some(ProviderRefreshGuard {
            controller: self,
            provider,
        })
    }
}

impl Drop for ProviderRefreshGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut active) = self.controller.active.lock() {
            active.remove(&self.provider);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::ProviderId;

    #[tokio::test]
    async fn provider_lock_allows_one_active_refresh() {
        let controller = RefreshController::default();

        let first = controller.try_begin_provider_refresh(ProviderId::Claude);
        let second = controller.try_begin_provider_refresh(ProviderId::Claude);

        assert!(first.is_some());
        assert!(second.is_none());
        drop(first);
        assert!(controller
            .try_begin_provider_refresh(ProviderId::Claude)
            .is_some());
    }
}
