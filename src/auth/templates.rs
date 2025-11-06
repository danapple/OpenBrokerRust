use yarte::Template;

#[derive(Template)]
#[template(path = "pages/welcome")]
pub struct Welcome<'a> {
    pub registration_failure_message: &'a str,
    pub registration_success_message: &'a str,
    pub login_failure_message: &'a str,
}
