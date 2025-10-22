pub(crate) enum Privilege {
    Owner,
    Read,
    Submit,
    Cancel,
}

#[derive(Clone)]
pub struct AccessControl {
}

impl AccessControl {
    pub fn new() -> AccessControl {
        AccessControl {}
    }
    pub fn is_allowed(& self , account_key: &String, customer_key: &String, privilege: Privilege) -> bool {
         account_key.eq("myaccount") && customer_key.eq("customer_key")
    }
}