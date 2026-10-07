use crate::infra::database::{
    diesel::{pool::DbPool, user_jti::DieselUserJtiRepository, users::DieselUserRepository},
    repositories::{Repositories, RepositoryProvider},
};

pub mod pool;
pub mod user_jti;
pub mod users;

pub struct DieselProvider;

impl RepositoryProvider for DieselProvider {
    type Users = DieselUserRepository;
    type UserJti = DieselUserJtiRepository;
}

impl Repositories<DieselProvider> {
    pub fn new(pool: &DbPool) -> Self {
        Self {
            users: DieselUserRepository::new(pool.clone()),
            user_jti: DieselUserJtiRepository::new(pool.clone()),
        }
    }
}
