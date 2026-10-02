use crate::api::Endpoint;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct Crate<'a> {
    name: Cow<'a, str>,
}

impl<'a> Crate<'a> {
    pub fn new(name: Cow<'a, str>) -> Self {
        Self { name }
    }
}

impl<'a> Endpoint for Crate<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}", self.name))
    }
}

#[cfg(test)]
mod tests {
    //
    // #[test]
    // fn id() {
    // }
}
