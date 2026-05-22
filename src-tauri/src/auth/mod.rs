//! Cross-platform credential storage abstraction.
//!
//! Production targets:
//! - macOS: Keychain via `keyring` crate
//! - Linux: libsecret (GNOME Keyring / KWallet)
//! - Windows: Credential Manager
//!
//! Browser cookie import is intentionally out of scope here; see Phase 2+ provider work.

mod credential_store;

pub use credential_store::{
    CredentialError, CredentialStore, DevCredentialStore, MemoryCredentialStore,
};
