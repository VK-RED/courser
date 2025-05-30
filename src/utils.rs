use argon2::{
    password_hash::{
        rand_core::OsRng, Error, PasswordHasher, SaltString
    }, Argon2, PasswordHash, PasswordVerifier
};

pub fn hash_password(password:&str)->Result<String, Error>{

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes(), salt.as_salt())?.to_string();
    Ok(password_hash)
}

pub fn verify_password(password:&str, hash:&str)->Result<(), Error>{

    let argon2 = Argon2::default(); 
    let parsed_hash = PasswordHash::new(&hash)?;
    argon2.verify_password(password.as_bytes(), &parsed_hash)?;

    Ok(())
}