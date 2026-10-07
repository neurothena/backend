use crate::infra::database::repositories::{user_jti::UserJtiRepository, users::UserRepository};

pub mod user_jti;
pub mod users;

pub trait RepositoryProvider {
    type Users: UserRepository + Send + Sync + 'static;
    type UserJti: UserJtiRepository + Send + Sync + 'static;
}

#[derive(Clone)]
pub struct Repositories<P: RepositoryProvider + 'static> {
    pub users: P::Users,
    pub user_jti: P::UserJti,
}
