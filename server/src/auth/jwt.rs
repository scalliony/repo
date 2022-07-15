use jsonwebtoken::*;

pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}
impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }

    pub fn encode<T: serde::Serialize>(&self, claims: &T) -> Result<String, errors::Error> {
        encode(&jsonwebtoken::Header::default(), claims, &self.encoding)
    }
    pub fn decode<T: serde::de::DeserializeOwned>(&self, jwt: &str) -> Result<T, errors::Error> {
        decode::<T>(jwt, &self.decoding, &jsonwebtoken::Validation::default()).map(|v| v.claims)
    }
}
