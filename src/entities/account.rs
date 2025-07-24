use argon2::{Argon2, password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, rand_core::OsRng, SaltString}};

pub struct Account {
    id: Option<i64>,
    pub hashed_password: String,
    pub display_name: String,
}

impl Account {
    pub fn new(id: Option<i64>, hashed_password: String, display_name: String) -> Account {
        Account {
            id,
            hashed_password,
            display_name,
        }
    }

    pub fn create( password: &str, display_name: &str) -> Account {
        Account {
            id: None,
            hashed_password: hash(password),
            display_name: display_name.to_string(),
        }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn matches_password(&self, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(&self.hashed_password).unwrap();
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }
}

fn hash(str: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default().hash_password(str.as_bytes(), &salt).unwrap().to_string()
}