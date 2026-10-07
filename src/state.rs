use std::sync::Arc;

use axum::extract::FromRef;

use crate::{
    config::Config,
    infra::{
        database::repositories::{Repositories, RepositoryProvider},
        jwt::service::JwtService,
    },
};

pub struct AppState<P: RepositoryProvider + 'static> {
    pub repositories: Arc<Repositories<P>>,
    pub config: Arc<Config>,
    pub jwt_service: JwtService,
}

impl<P: RepositoryProvider> AppState<P> {
    pub fn new(repos: Repositories<P>, config: Arc<Config>) -> Self {
        Self {
            repositories: Arc::new(repos),
            jwt_service: JwtService::new(&config),
            config,
        }
    }
}

impl<P: RepositoryProvider + 'static> Clone for AppState<P> {
    fn clone(&self) -> Self {
        Self {
            jwt_service: self.jwt_service.clone(),
            repositories: self.repositories.clone(),
            config: self.config.clone(),
        }
    }
}

impl<P: RepositoryProvider> FromRef<AppState<P>> for Arc<Config> {
    fn from_ref(state: &AppState<P>) -> Self {
        state.config.clone()
    }
}

impl<P: RepositoryProvider> FromRef<AppState<P>> for JwtService {
    fn from_ref(state: &AppState<P>) -> Self {
        state.jwt_service.clone()
    }
}

impl<P: RepositoryProvider> FromRef<AppState<P>> for Arc<Repositories<P>> {
    fn from_ref(state: &AppState<P>) -> Self {
        state.repositories.clone()
    }
}
