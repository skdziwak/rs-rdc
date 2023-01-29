use genco::prelude::*;
use genco::quote;

pub trait Compact<T> {
    fn compact(&self) -> T;
    fn join(&self, sep: &str) -> T;
}

impl Compact<java::Tokens> for Vec<java::Tokens> {
    fn compact(&self) -> java::Tokens {
        let iter = self.iter();
        quote!($(for t in iter => $t))
    }

    fn join(&self, sep: &str) -> java::Tokens {
        let iter = self.iter();
        quote!($(for t in iter join ($sep) => $t))
    }
}

#[macro_export]
macro_rules! quote_iter {
    ($val:expr => $t:expr) => {{
        $val.map($t)
            .collect::<Vec<genco::prelude::java::Tokens>>()
            .compact()
    }};

    ($val:expr, join($sep:expr) => $t:expr) => {{
        $val.map($t)
            .collect::<Vec<genco::prelude::java::Tokens>>()
            .join($sep)
    }};
}
