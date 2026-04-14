// use sea_orm::prelude::Uuid;
// use serde::{Deserialize, Serialize};
// use validator::Validate;

// #[derive(Debug, Deserialize, Validate)]
// pub struct LoginPayload {
//     #[validate(email(message = "Некорректный формат email"))]
//     pub email: String,
//     #[validate(length(min = 8, message = "Пароль должен быть не менее 8 символов"))]
//     pub password: String,
// }
// #[derive(Debug, Deserialize, Serialize, Validate)]
// pub struct RegisterPayload {
//     #[validate(email(message = "Некорректный формат email"))]
//     pub email: String,
//     #[validate(length(min = 8, message = "Пароль должен быть не менее 8 символов"))]
//     pub password: String,
//     pub last_name: String,
//     pub first_name: String,
//     pub study_group: Option<String>,
//     pub telephone: Option<String>,
// }
// #[derive(Debug, Deserialize, Validate)]
// pub struct PasswordResetPayload {
//     pub id: Uuid,
//     #[validate(length(equal = 6, message = "Код должен состоять из 6 цифр"))]
//     pub code: String,
//     #[validate(length(min = 8, message = "Пароль должен быть не менее 8 символов"))]
//     pub password: String,
// }
// #[derive(Debug, Deserialize, Validate)]
// pub struct EmailResetPayload {
//     pub id: Uuid,
//     #[validate(length(equal = 6, message = "Код должен состоять из 6 цифр"))]
//     pub code: String,
// }
