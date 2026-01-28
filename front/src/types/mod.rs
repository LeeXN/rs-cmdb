// 使用公共模块中的类型
pub use common::entity::dictionary::Dictionary;
pub use common::entity::hardware::Hardware;
pub use common::entity::hardware::NICStatus;
pub use common::entity::hardware::StorageType;
pub use common::entity::user::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, Role, UpdateUserRequest,
    UserResponse as User,
};
pub use common::models::*;
