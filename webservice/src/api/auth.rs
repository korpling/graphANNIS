use crate::AppState;
use actix_web::{post, web};
use serde::Deserialize;

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub username_or_email: String,
    pub password: String,
}

#[post("/login")]
async fn login(login_data: web::Json<LoginData>, state: web::Data<AppState>) -> String {
    // Check if user ID is set in configuration
    let provided_user = &login_data.username_or_email;
    if let Some(user) = state.settings.users.get(provided_user) {
        // Add Salt to password, calculate hash and compare against our settings
        let provided_password = &login_data.password;
        if bcrypt::verify(&provided_password, &user.password).unwrap() {
            format!("YES, we know you {}.", &provided_user)
        } else {
            format!("You are not you!")
        }
    } else {
        format!("Unknown")
    }
}
