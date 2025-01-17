mod err;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub use err::*;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Claims {
    pub id: i32,
    pub email: String,
    pub iss: String,
    pub exp: usize,
}

pub struct Jwt {
    pub secret: String,
    pub exp: i64,
    pub iss: String,
}

impl Jwt {
    pub fn new(secret: String, exp: i64, iss: String) -> Self {
        Self { secret, exp, iss }
    }

    pub fn new_claims(&self, id: i32, email: String) -> Claims {
        Claims {
            id,
            email,
            iss: self.iss.to_string(),
            exp: self.calc_claims_exp(),
        }
    }

    fn calc_claims_exp(&self) -> usize {
        (Utc::now() + Duration::seconds(self.exp)).timestamp_millis() as usize
    }

    fn secret_bytes(&self) -> &[u8] {
        (&self.secret).as_bytes()
    }

    pub fn token(&self, claims: &Claims) -> Result<String, crate::Error> {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.secret_bytes()),
        )
        .map_err(crate::Error::from)
    }
    
    pub fn verify_and_get(&self, token: &str) -> Result<Claims, crate::Error> {
        let mut v = Validation::new(jsonwebtoken::Algorithm::HS256);
        v.set_issuer(&[self.iss.clone()]);
        let token_data = decode(token, &DecodingKey::from_secret(self.secret_bytes()), &v)
            .map_err(crate::Error::from)?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use crate::Jwt;

    
    const SECRET: &str = "blog";
    const ISS: &str = "blog";
    
    #[test]
    fn test_gen_token() {
        let jwt = Jwt::new(SECRET.to_string(), 120, ISS.to_string());
        let claims = jwt.new_claims(1, "cakeal@qq.com".to_string());
        let token = jwt.token(&claims).unwrap();
        println!("{:?}", token);
    }
    
    #[test]
    fn test_get_cliams() {
        let jwt = Jwt::new(SECRET.to_string(), 120, ISS.to_string());
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6MSwiZW1haWwiOiJjYWtlYWxAcXEuY29tIiwiaXNzIjoiYmxvZyIsImV4cCI6MTcyMTk4MTE2Mjc3N30.2dDMtPV_XJLCJDhou44ST8FslhDe_sZgXV_uH-2WCbg";
        let claims = jwt.verify_and_get(token).unwrap();
        println!("{:?}", claims);
    }
}