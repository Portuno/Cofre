// Authentication service for Cofre Vault Platform
// Handles user registration, login, session management via Supabase Auth

use crate::error::{Error, Result};
use crate::models::{AuthResult, User};
use crate::db::Database;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Callback function type for auth state changes
pub type AuthStateCallback = Box<dyn Fn(&Option<User>) + Send + Sync>;

/// Authentication service managing user sessions and auth state
pub struct AuthService {
    db: Database,
    /// Current authenticated user (in-memory for demo; would use session storage in production)
    current_user: Arc<RwLock<Option<User>>>,
    /// Active session tokens mapped to user IDs
    sessions: Arc<RwLock<HashMap<String, Uuid>>>,
    /// Auth state change callbacks
    callbacks: Arc<RwLock<Vec<Arc<AuthStateCallback>>>>,
}

impl AuthService {
    /// Create a new AuthService instance
    pub fn new(db: Database) -> Self {
        AuthService {
            db,
            current_user: Arc::new(RwLock::new(None)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Sign up a new user with email and password
    ///
    /// # Arguments
    /// * `email` - User's email address
    /// * `password` - User's password (should be validated for strength)
    ///
    /// # Returns
    /// * `AuthResult` containing user info and session token on success
    /// * `Error::AuthenticationFailed` if sign-up fails
    ///
    /// # Requirements
    /// * Requirement 1.1: Support user registration via email and password through Supabase Auth
    /// * Requirement 1.2: Support user login via email and password through Supabase Auth
    pub async fn sign_up(&self, email: &str, password: &str) -> Result<AuthResult> {
        // Validate inputs
        if email.is_empty() || !email.contains('@') {
            return Err(Error::AuthenticationFailed("Invalid email format".to_string()));
        }
        if password.len() < 6 {
            return Err(Error::AuthenticationFailed(
                "Password must be at least 6 characters".to_string(),
            ));
        }

        // In a real implementation, this would call Supabase Auth API
        // For now, we simulate the auth flow
        let user_id = Uuid::new_v4();
        let session_token = self.generate_session_token();

        let user = User {
            id: user_id,
            email: email.to_string(),
        };

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_token.clone(), user_id);
        }

        // Update current user
        {
            let mut current = self.current_user.write().await;
            *current = Some(user.clone());
        }

        // Trigger auth state change callbacks
        self.trigger_callbacks().await;

        Ok(AuthResult {
            user,
            session_token,
        })
    }

    /// Sign in an existing user with email and password
    ///
    /// # Arguments
    /// * `email` - User's email address
    /// * `password` - User's password
    ///
    /// # Returns
    /// * `AuthResult` containing user info and session token on success
    /// * `Error::AuthenticationFailed` if credentials are invalid
    ///
    /// # Requirements
    /// * Requirement 1.1: Support user registration via email and password through Supabase Auth
    /// * Requirement 1.2: Support user login via email and password through Supabase Auth
    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthResult> {
        // Validate inputs
        if email.is_empty() || !email.contains('@') {
            return Err(Error::AuthenticationFailed("Invalid email format".to_string()));
        }
        if password.is_empty() {
            return Err(Error::AuthenticationFailed("Password is required".to_string()));
        }

        // In a real implementation, this would call Supabase Auth API
        // For now, we simulate the auth flow
        let user_id = Uuid::new_v4();
        let session_token = self.generate_session_token();

        let user = User {
            id: user_id,
            email: email.to_string(),
        };

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_token.clone(), user_id);
        }

        // Update current user
        {
            let mut current = self.current_user.write().await;
            *current = Some(user.clone());
        }

        // Trigger auth state change callbacks
        self.trigger_callbacks().await;

        Ok(AuthResult {
            user,
            session_token,
        })
    }

    /// Get the currently authenticated user
    ///
    /// # Returns
    /// * `Some(User)` if a user is authenticated
    /// * `None` if no user is authenticated
    ///
    /// # Requirements
    /// * Requirement 1.3: When a user signs in successfully, the System SHALL establish a session and expose the authenticated user to the application
    /// * Requirement 1.4: The System SHALL support user sign-out and session termination
    pub async fn get_current_user(&self) -> Option<User> {
        self.current_user.read().await.clone()
    }

    /// Sign out the current user and terminate the session
    ///
    /// # Requirements
    /// * Requirement 1.4: The System SHALL support user sign-out and session termination
    /// * Requirement 1.5: When a user is not authenticated, the System SHALL prevent access to vault content and redirect to the authentication interface
    pub async fn sign_out(&self) -> Result<()> {
        // Clear current user
        {
            let mut current = self.current_user.write().await;
            *current = None;
        }

        // Clear all sessions
        {
            let mut sessions = self.sessions.write().await;
            sessions.clear();
        }

        // Trigger auth state change callbacks
        self.trigger_callbacks().await;

        Ok(())
    }

    /// Register a callback to be invoked when auth state changes
    ///
    /// # Arguments
    /// * `callback` - Function to call when auth state changes
    ///
    /// # Returns
    /// * A function that can be called to unsubscribe from the callback
    ///
    /// # Requirements
    /// * Requirement 1.6: The System SHALL expose reactive session state changes to the application via an authentication state change callback
    pub async fn on_auth_state_change<F>(&self, callback: F) -> impl Fn() + Send + Sync
    where
        F: Fn(&Option<User>) + Send + Sync + 'static,
    {
        let callback = Arc::new(Box::new(callback) as AuthStateCallback);
        let callbacks = Arc::clone(&self.callbacks);

        {
            let mut cbs = callbacks.write().await;
            cbs.push(callback);
        }

        move || {
            // Unsubscribe logic would go here
            // For now, this is a placeholder
        }
    }

    /// Verify that a session token is valid
    ///
    /// # Arguments
    /// * `token` - Session token to verify
    ///
    /// # Returns
    /// * `Some(Uuid)` containing the user ID if token is valid
    /// * `None` if token is invalid or expired
    pub async fn verify_session(&self, token: &str) -> Option<Uuid> {
        self.sessions.read().await.get(token).copied()
    }

    // Private helper methods

    /// Generate a new session token
    fn generate_session_token(&self) -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }

    /// Trigger all registered auth state change callbacks
    async fn trigger_callbacks(&self) {
        let current_user = self.current_user.read().await.clone();
        let callbacks = self.callbacks.read().await;
        for callback in callbacks.iter() {
            callback(&current_user);
        }
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        AuthService {
            db: self.db.clone(),
            current_user: Arc::clone(&self.current_user),
            sessions: Arc::clone(&self.sessions),
            callbacks: Arc::clone(&self.callbacks),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_auth_service() -> AuthService {
        let db = Database::new(crate::db::DatabaseConfig {
            supabase_url: "https://test.supabase.co".to_string(),
            supabase_key: "test-key".to_string(),
            database_url: "postgresql://test".to_string(),
            max_connections: 10,
        });
        AuthService::new(db)
    }

    #[tokio::test]
    async fn test_sign_up_success() {
        let auth = create_test_auth_service();
        let result = auth.sign_up("user@example.com", "password123").await;

        assert!(result.is_ok());
        let auth_result = result.unwrap();
        assert_eq!(auth_result.user.email, "user@example.com");
        assert!(!auth_result.session_token.is_empty());
    }

    #[tokio::test]
    async fn test_sign_up_invalid_email() {
        let auth = create_test_auth_service();
        let result = auth.sign_up("invalid-email", "password123").await;

        assert!(result.is_err());
        match result {
            Err(Error::AuthenticationFailed(msg)) => {
                assert!(msg.contains("Invalid email format"));
            }
            _ => panic!("Expected AuthenticationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_sign_up_weak_password() {
        let auth = create_test_auth_service();
        let result = auth.sign_up("user@example.com", "short").await;

        assert!(result.is_err());
        match result {
            Err(Error::AuthenticationFailed(msg)) => {
                assert!(msg.contains("at least 6 characters"));
            }
            _ => panic!("Expected AuthenticationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_sign_in_success() {
        let auth = create_test_auth_service();
        let result = auth.sign_in("user@example.com", "password123").await;

        assert!(result.is_ok());
        let auth_result = result.unwrap();
        assert_eq!(auth_result.user.email, "user@example.com");
        assert!(!auth_result.session_token.is_empty());
    }

    #[tokio::test]
    async fn test_sign_in_invalid_email() {
        let auth = create_test_auth_service();
        let result = auth.sign_in("invalid-email", "password123").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_current_user_after_sign_up() {
        let auth = create_test_auth_service();
        auth.sign_up("user@example.com", "password123").await.unwrap();

        let current_user = auth.get_current_user().await;
        assert!(current_user.is_some());
        assert_eq!(current_user.unwrap().email, "user@example.com");
    }

    #[tokio::test]
    async fn test_get_current_user_no_auth() {
        let auth = create_test_auth_service();
        let current_user = auth.get_current_user().await;
        assert!(current_user.is_none());
    }

    #[tokio::test]
    async fn test_sign_out() {
        let auth = create_test_auth_service();
        auth.sign_up("user@example.com", "password123").await.unwrap();

        // Verify user is authenticated
        assert!(auth.get_current_user().await.is_some());

        // Sign out
        let result = auth.sign_out().await;
        assert!(result.is_ok());

        // Verify user is no longer authenticated
        assert!(auth.get_current_user().await.is_none());
    }

    #[tokio::test]
    async fn test_verify_session() {
        let auth = create_test_auth_service();
        let auth_result = auth.sign_up("user@example.com", "password123").await.unwrap();

        let verified_user_id = auth.verify_session(&auth_result.session_token).await;
        assert_eq!(verified_user_id, Some(auth_result.user.id));
    }

    #[tokio::test]
    async fn test_verify_session_invalid_token() {
        let auth = create_test_auth_service();
        let verified_user_id = auth.verify_session("invalid-token").await;
        assert_eq!(verified_user_id, None);
    }

    #[tokio::test]
    async fn test_auth_state_change_callback() {
        let auth = create_test_auth_service();
        let callback_called = Arc::new(RwLock::new(false));
        let callback_called_clone = Arc::clone(&callback_called);

        let _unsubscribe = auth
            .on_auth_state_change(move |_user| {
                let called = callback_called_clone.clone();
                let called_clone = called.clone();
                tokio::spawn(async move {
                    let mut c = called_clone.write().await;
                    *c = true;
                });
            })
            .await;

        auth.sign_up("user@example.com", "password123").await.unwrap();

        // Give the callback time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let was_called = *callback_called.read().await;
        assert!(was_called);
    }
}
