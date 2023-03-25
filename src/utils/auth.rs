use argon2::password_hash::{
    errors::Error, rand_core::{self, RngCore}, ParamsString,
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, Output};
use lazy_static::lazy_static;
use hex;
use std::env;

const ALGO: argon2::Algorithm = argon2::Algorithm::Argon2id;
const VER: argon2::Version = argon2::Version::V0x13;
const HASH_LEN: usize = 64;
const SALT_LEN: usize = 48;

lazy_static! {
    static ref SECRET: Vec<u8> = hex::decode(
        env::var("PASS_SECRET").expect("PASS_SECRET env var should be set")
    ).expect("secret env var cannot be decoded into bytes");

    static ref A2: argon2::Argon2<'static> = argon2::Argon2::new_with_secret(
        &SECRET.as_slice(),
        ALGO,
        VER,
        argon2::Params::new(
            4096,
            12,
            4,
            Some(HASH_LEN)
        ).expect("argon2 hasher params valid input")
    ).expect("argon2 hasher creation failure");
}

pub(crate) type AuthError = Error;
pub(crate) const PASSWORD_ERROR: AuthError = Error::Password;

pub(crate) struct Hashed {
    pub hash: [u8; HASH_LEN],
    pub salt: [u8; SALT_LEN]
}

impl Hashed {
    pub fn new(hash: [u8; HASH_LEN], salt: [u8; SALT_LEN]) -> Self {
        Self {
            hash,
            salt
        }
    }

    pub fn new_check_length(hash: &[u8], salt: &[u8]) -> Self {
        Self {
            hash: hash.try_into().expect(
                &format!("Incorrect hash length! Expected length {}, got length {}",
                         HASH_LEN, hash.len())),
            salt: salt.try_into().expect(
                &format!("Incorrect salt length! Expected length {}, got length {}",
                         SALT_LEN, salt.len()))
        }
    }

    pub fn get_salt_string(&self) -> Result<SaltString, AuthError> {
        SaltString::encode_b64(&self.salt)
    }

    pub fn as_password_hash<'a>(&'a self, salt_str: &'a SaltString) -> Result<PasswordHash<'a>, AuthError> {
        Ok(PasswordHash {
            algorithm: ALGO.into(),
            version: Some(VER.into()),
            params: ParamsString::try_from(A2.params())?,
            salt: Some(salt_str.as_salt()),
            hash: Some(Output::new(&self.hash[..])?),
        })
    }
}

pub(crate) fn hash(password: &str) -> Result<Hashed, Error> {
    let mut salt_bytes = [0u8; SALT_LEN];
    rand_core::OsRng.fill_bytes(&mut salt_bytes);
    let salt_str = SaltString::encode_b64(&salt_bytes)?;
    let p = A2.hash_password(password.as_bytes(), salt_str.as_salt())?;
    Ok(Hashed::new(
        p.hash.expect("hash has output").as_bytes()
            .try_into().expect("hash length is set"),
        salt_bytes
    ))
}

pub(crate) fn verify(password: String, hash: Hashed) -> Result<(), AuthError> {
    let salt_str = hash.get_salt_string()?;
    A2.verify_password(password.as_bytes(), &hash.as_password_hash(&salt_str)?)
}
