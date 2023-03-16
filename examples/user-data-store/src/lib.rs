/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An example of a user data store that with an async interface that's driven by the executor on
//! the foreign side of the FFI.  This is achieved by awaiting async callback interface methods.
//! See the callbacks section of the manual for the technical details.

use async_once_cell::OnceCell;
use async_mutex::{Mutex, MutexGuard};
use futures::{join, FutureExt};
use std::sync::Arc;
use uniffi::{CallbackResult, ThreadQueue, ForeignFuture};

#[derive(uniffi::Error)]
pub enum Error {
    DatabaseError(String),
    UnexpectedError(String),
}

impl From<uniffi::UnexpectedCallbackError> for Error {
    fn from(_: uniffi::UnexpectedCallbackError) -> Self {
        Self::UnexpectedError("Error invoking callback interface")
    }
}

pub type Result<T> = Result<T, Error>;

#[uniffi::callback_interface]
pub trait Logger {
    /// Log a message
    async fn log(&self, message: String) -> CallbackResult<()>;
}

#[uniffi::callback_interface]
pub trait CryptoKeyStore {
    /// Get a key for cryptography operations.  Depending on the application, this may load the key
    /// from a file or from an OS key store and may require the user to take some action to unloke
    /// the key.
    async fn get(&self) -> CallbackResult<String>;
}

/// Event notification system.
#[uniffi::callback_interface]
pub trait EventListener {
    async fn db_operation_complete(&self, store: Arc<UserStore>, success: bool) -> CallbackResult<()>;
}

#[derive(Debug, Default, Clone)]
pub struct UserStore {
    logger: Box<dyn Logger>,
    key_store: Box<dyn KeyStore>,
    listener: Box<dyn EventListener>,
    queue: ThreadQueue,
    // Use OnceCell lazily initialize the DB/crypto key.
    // async_once_cell also has a `Lazy` type, which is closer to what we really want, but it seems
    // to require spelling out the concrete type that implement Future, which would be awkward.
    db_cell: OnceCell<CallbackResult<Mutex<UserDatabase>>>,
    crypto_key_cell: OnceCell<CallbackResult<String>>,
}


#[uniffi::export]
impl UserStore {
    // Constructors are not yet implemented with proc-macros, but let's pretend they are
    #[constructor]
    pub fn new(logger: Box<dyn Logger>, key_store: Box<dyn Logger>, listener: Box<dyn EventListener>, queue: ThreadQueue) -> Self {
        Self {
            logger,
            key_store,
            listener,
            queue,
            db_cell: OnceCell::new(),
            crypto_key_cell: OnceCell::new()
        }
    }

    async fn get_db(&self) -> Result<&Mutex<UserDatabase>> {
        Ok(db_cell.get_or_init(self.queue.schedule(|| Mutex::new(UserDatabase::new()))).await?)
    }

    async fn get_crypto_key(&self) -> Result<&str> {
        Ok(db_cell.get_or_init(self.key_store.get()).await?)
    }

    /// Warmup by loading the database
    ///
    /// TODO: use this method as an example to show how the plumbing works and how execution flows.
    /// I tried writing it up, but it was hard to do without actual code to use as a reference.
    pub async fn warmup(&self) {
        self.get_db().await;
    }

    // Wrapper for a database operation that awaits on DB initialization and loading the crypto
    // key.  f should be a synchronous function.
    //
    // This is the main point of this example.  This wrapper uses async callback interfaces to:
    //   - Initialize the database and load the crypto key asynchronously
    //   - Schedule the database operation in a background thread
    //   - Log any errors
    //   - Notify the application that the operation completed
    fn db_operation<T, F: FnOnce(MutexGuard<'_, Database>, &str) -> Result<T>>(self: &Arc<Self>, f: F) -> Result<T> {
        let db = self.get_db().await?;
        let crypto_key = self.get_crypto_key().await?;
        let result = self.queue.schedule(|| f(db, crypto_key)).await?;
        if let Err(e) = &result {
            self.logger.log(format!("Database error: {e}")).fire();
        }
        self.events.db_operation_complete(self.clone(), result.is_ok()).fire();
        result
    }

    pub async fn add_user(self: Arc<Self>, username: String, password: String) -> Result<()> {
        &self.db_operation(|db, key| {
            db.insert(username, encrypt(password, key))
        })
    }

    pub async fn lookup_password(self: Arc<Self>, username: String, password: String) -> Result<String> {
        &self.db_operation(|db, key| {
            Ok(decrypt(db.lookup(username)?, key))
        })
    }

    pub async fn delete_user(self: Arc<Self>, username: String, password: String) -> Result<()> {
        &self.db_operation(|db, _| {
            db.delete(username)
        })
    }
}

include!(concat!(env!("OUT_DIR"), "/user_store.uniffi.rs"));
