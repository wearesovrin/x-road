use common::{XRoadError, XRoadResult};
use cryptoki::context::{CInitializeArgs, Pkcs11};
use cryptoki::mechanism::Mechanism;
use cryptoki::object::{Attribute, ObjectClass};
use cryptoki::session::{Session, UserType};
use cryptoki::types::AuthPin;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Hardware Security Module (HSM) token manager using standard PKCS#11 C-API.
#[derive(Clone)]
pub struct HsmTokenManager {
    module_path: String,
    slot_id: u64,
    pkcs11: Arc<Pkcs11>,
    session: Arc<Mutex<Option<Session>>>,
}

impl HsmTokenManager {
    /// Attempts to load the vendor PKCS#11 shared library and open a session with the HSM.
    pub fn new(
        module_path: &str,
        pin: Option<&str>,
        slot_index: Option<usize>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if !Path::new(module_path).exists() {
            return Err(format!("PKCS#11 module not found at: {module_path}").into());
        }

        let pkcs11 = Pkcs11::new(module_path)?;
        pkcs11.initialize(CInitializeArgs::OsThreads)?;

        let slots = pkcs11.get_slots_with_token()?;
        if slots.is_empty() {
            return Err("No active PKCS#11 slots with token found".into());
        }

        let idx = slot_index.unwrap_or(0).min(slots.len() - 1);
        let slot = slots[idx];

        let session = if pin.is_some() {
            pkcs11.open_rw_session(slot)?
        } else {
            pkcs11.open_ro_session(slot)?
        };

        if let Some(user_pin) = pin {
            let auth_pin = AuthPin::new(user_pin.to_string().into());
            session.login(UserType::User, Some(&auth_pin))?;
        }

        Ok(Self {
            module_path: module_path.to_string(),
            slot_id: slot.into(),
            pkcs11: Arc::new(pkcs11),
            session: Arc::new(Mutex::new(Some(session))),
        })
    }

    /// Automatically attempts to connect to an HSM from environment variables or returns None.
    pub fn from_env() -> Option<Self> {
        let module_path = std::env::var("XROAD_PKCS11_MODULE_PATH").ok()?;
        let pin = std::env::var("XROAD_PKCS11_PIN").ok();
        let slot = std::env::var("XROAD_PKCS11_SLOT")
            .ok()
            .and_then(|s| s.parse().ok());

        match Self::new(&module_path, pin.as_deref(), slot) {
            Ok(mgr) => {
                tracing::info!(
                    module = %module_path,
                    "Successfully initialized hardware PKCS#11 token interface"
                );
                Some(mgr)
            }
            Err(e) => {
                tracing::warn!(
                    module = %module_path,
                    "Failed to initialize PKCS#11 hardware token (falling back to software token): {}",
                    e
                );
                None
            }
        }
    }

    /// Signs a precomputed digest using the designated PKCS#11 key and mechanism.
    pub async fn sign(&self, key_label: &str, data: &[u8]) -> XRoadResult<Vec<u8>> {
        let session_guard = self.session.lock().await;
        let session = session_guard
            .as_ref()
            .ok_or_else(|| XRoadError::Crypto("PKCS#11 session is closed".to_string()))?;

        let template = vec![
            Attribute::Class(ObjectClass::PRIVATE_KEY),
            Attribute::Label(key_label.as_bytes().to_vec()),
        ];

        let objects = session
            .find_objects(&template)
            .map_err(|e| XRoadError::Crypto(format!("PKCS#11 find_objects failed: {e}")))?;

        let key_handle = objects
            .first()
            .copied()
            .ok_or_else(|| XRoadError::Crypto(format!("PKCS#11 key not found: {key_label}")))?;

        // Default to ECDSA or RSA PKCS mechanism
        let mechanism = Mechanism::Ecdsa;
        let signature = session
            .sign(&mechanism, key_handle, data)
            .map_err(|e| XRoadError::Crypto(format!("PKCS#11 sign operation failed: {e}")))?;

        Ok(signature)
    }

    pub fn module_path(&self) -> &str {
        &self.module_path
    }

    pub fn slot_id(&self) -> u64 {
        self.slot_id
    }

    pub fn pkcs11(&self) -> &Arc<Pkcs11> {
        &self.pkcs11
    }
}
