#[derive(Clone, Copy)]
pub enum LoginKind {
    User,
    Admin,
}

pub fn process_login(kind: LoginKind) {
    println!("Processing login...");
    let status = match kind {
        LoginKind::User => "active",
        LoginKind::Admin => "super_admin",
    };
    save(status);
}

fn save(s: &str) {
    println!("Saving {s}");
}

// Proxies for backwards compatibility
pub fn process_user_login() {
    process_login(LoginKind::User);
}

pub fn process_admin_login() {
    process_login(LoginKind::Admin);
}