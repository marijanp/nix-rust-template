use askama::Template;

#[derive(Template)]
#[template(path = "login-button.html")]
pub struct LoginButton;

#[derive(Template)]
#[template(path = "logout-button.html")]
pub struct LogoutButton;

#[derive(Template)]
#[template(path = "error-message.html")]
pub struct ErrorMessage<'a> {
    pub message: &'a str,
}
