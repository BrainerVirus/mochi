use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::core::models::ProviderId;

#[derive(Default)]
pub struct RefreshController {
    providers: Mutex<HashMap<ProviderId, Arc<Mutex<()>>>>,
}

impl RefreshController {
    pub async fn begin_provider_refresh(&self, provider: ProviderId) -> OwnedMutexGuard<()> {
        let provider_lock = {
            let mut providers = self.providers.lock().await;
            Arc::clone(
                providers
                    .entry(provider)
                    .or_insert_with(|| Arc::new(Mutex::new(()))),
            )
        };
        provider_lock.lock_owned().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::ProviderId;

    #[tokio::test]
    async fn same_provider_refresh_waits_for_active_refresh() {
        let controller = RefreshController::default();
        let first = controller.begin_provider_refresh(ProviderId::Claude).await;
        let mut second = Box::pin(controller.begin_provider_refresh(ProviderId::Claude));

        tokio::select! {
            biased;
            _ = &mut second => panic!("same-provider refresh must wait"),
            () = std::future::ready(()) => {}
        }

        drop(first);
        let _second = second.await;
    }

    #[tokio::test]
    async fn different_provider_refreshes_remain_independent() {
        let controller = RefreshController::default();
        let _claude = controller.begin_provider_refresh(ProviderId::Claude).await;
        let mut cursor = Box::pin(controller.begin_provider_refresh(ProviderId::Cursor));

        tokio::select! {
            biased;
            _cursor = &mut cursor => {}
            () = std::future::ready(()) => panic!("different providers must not block each other"),
        }
    }
}
