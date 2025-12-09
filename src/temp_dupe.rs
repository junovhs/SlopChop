pub fn process_user_login() {
    println!("Processing login...");
    let status = "active";
    save(status);
}

pub fn process_admin_login() {
    println!("Processing login...");
    let status = "super_admin";
    save(status);
}

fn save(s: &str) {
    println!("Saving {s}");
}
