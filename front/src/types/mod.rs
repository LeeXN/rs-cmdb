// 使用公共模块中的类型
pub use common::models::*;
pub use common::entity::hardware::Hardware;
pub use common::entity::hardware::NICStatus;
pub use common::entity::hardware::StorageType;
pub use common::entity::user::{Role, LoginRequest, LoginResponse, UserResponse as User, CreateUserRequest, UpdateUserRequest, ChangePasswordRequest};
pub use common::entity::dictionary::Dictionary;