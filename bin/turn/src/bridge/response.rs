use serde::Deserialize;
use std::convert::TryFrom;
use anyhow::{
    Result,
    anyhow
};

/// auth response struct.
#[derive(Deserialize)]
pub struct Auth {
    pub password: String,
    pub group: u32,
}

/// response from nats request.
///
/// data is empty when error is not empty.
#[derive(Deserialize)]
pub struct Response<T> {
    pub error: Option<String>,
    pub data: Option<T>
}

impl<'a, T: Deserialize<'a>> TryFrom<&'a [u8]> for Response<T> {
    /// # Example
    ///
    /// ```no_run
    /// let res_buf = [
    ///     0x7b, 0x22, 0x65, 0x72, 0x72, 
    ///     0x6f, 0x72, 0x22, 0x3a, 0x6e,
    ///     0x75, 0x6c, 0x6c, 0x2c, 0x22, 
    ///     0x64, 0x61, 0x74, 0x61, 0x22,
    ///     0x3a, 0x7b, 0x22, 0x67, 0x72,
    ///     0x6f, 0x75, 0x70, 0x22, 0x3a,
    ///     0x30, 0x2c, 0x22, 0x70, 0x61,
    ///     0x73, 0x73, 0x77, 0x6f, 0x72,
    ///     0x64, 0x22, 0x3a, 0x22, 0x70,
    ///     0x61, 0x6e, 0xx64, 0x61, 0x22,
    ///     0x7d, 0x7d
    /// ];
    ///
    /// // Response<Auth>::try_from(&res_buf[..]).unwrap()
    /// ```
    type Error = anyhow::Error;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(value)?)
    }
}

impl<T> Response<T> {
    /// into Result from Response.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let res_buf = [
    ///     0x7b, 0x22, 0x65, 0x72, 0x72, 
    ///     0x6f, 0x72, 0x22, 0x3a, 0x6e,
    ///     0x75, 0x6c, 0x6c, 0x2c, 0x22, 
    ///     0x64, 0x61, 0x74, 0x61, 0x22,
    ///     0x3a, 0x7b, 0x22, 0x67, 0x72,
    ///     0x6f, 0x75, 0x70, 0x22, 0x3a,
    ///     0x30, 0x2c, 0x22, 0x70, 0x61,
    ///     0x73, 0x73, 0x77, 0x6f, 0x72,
    ///     0x64, 0x22, 0x3a, 0x22, 0x70,
    ///     0x61, 0x6e, 0xx64, 0x61, 0x22,
    ///     0x7d, 0x7d
    /// ];
    ///
    /// let res = Response<Auth>::try_from(&res_buf[..])
    ///     .unwrap()
    ///     .into_result()
    ///     .unwrap();
    /// // res.password
    /// ```
    pub fn into_result(self) -> Result<T> {
        match self.error {
            Some(e) => Err(anyhow!(e)),
            None => match self.data {
                None => Err(anyhow!("bad response!")),
                Some(a) => Ok(a)
            }
        }
    }
}
