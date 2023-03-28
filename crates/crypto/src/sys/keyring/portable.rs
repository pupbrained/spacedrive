use std::{collections::HashMap, sync::Mutex};

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{
	crypto::{Decryptor, Encryptor},
	hashing::Hasher,
	types::{Aad, Algorithm, DerivationContext, Key, Nonce, Salt},
	Error, Protected, Result,
};

use super::{Identifier, KeyringInterface, KeyringName};

pub const PORTABLE_KEYRING_CONTEXT: DerivationContext =
	DerivationContext::new("crypto 2023-03-27 21:37:42 portable keyring context");

// Ephemeral, session-only
pub struct PortableKeyring {
	key: Key,
	inner: Mutex<HashMap<String, PortableKeyringItem>>,
}

#[derive(Zeroize, ZeroizeOnDrop)]
struct PortableKeyringItem(#[zeroize(skip)] Salt, #[zeroize(skip)] Nonce, Vec<u8>);

impl KeyringInterface for PortableKeyring {
	fn new() -> Result<Self> {
		let s = Self {
			key: Key::generate(),
			inner: Mutex::new(HashMap::new()),
		};

		Ok(s)
	}

	fn contains_key(&self, id: &Identifier) -> bool {
		self.inner
			.lock()
			.map_err(|_| Error::KeyringError)
			.map_or(false, |x| x.contains_key(&id.hash()))
	}

	fn get(&self, id: &Identifier) -> Result<Protected<String>> {
		let handle = self.inner.lock().map_err(|_| Error::KeyringError)?;
		let item = handle.get(&id.hash()).ok_or(Error::KeyringError)?;

		let value = Decryptor::decrypt_tiny(
			Hasher::derive_key(self.key.clone(), item.0, PORTABLE_KEYRING_CONTEXT),
			item.1,
			Algorithm::default(),
			&item.2,
			Aad::Null,
		)?;

		String::from_utf8(value.into_inner())
			.map(Protected::new)
			.map_err(|_| Error::KeyringError)
	}

	fn insert(&self, id: &Identifier, value: Protected<String>) -> Result<()> {
		let salt = Salt::generate();
		let nonce = Nonce::generate(Algorithm::default());

		let bytes = Encryptor::encrypt_tiny(
			Hasher::derive_key(self.key.clone(), salt, PORTABLE_KEYRING_CONTEXT),
			nonce,
			Algorithm::default(),
			value.expose().as_bytes(),
			Aad::Null,
		)?;

		let item = PortableKeyringItem(salt, nonce, bytes);

		self.inner
			.lock()
			.map_err(|_| Error::KeyringError)?
			.insert(id.hash(), item);

		Ok(())
	}

	fn remove(&self, id: &Identifier) -> Result<()> {
		self.inner
			.lock()
			.map_err(|_| Error::KeyringError)?
			.remove(&id.hash());

		Ok(())
	}

	fn name(&self) -> KeyringName {
		KeyringName::Portable
	}
}
