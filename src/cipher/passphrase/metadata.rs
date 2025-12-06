use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! metadata_struct {
    ($version:ident, { $($field:ident : $ty:ty),* $(,)? }) => {
        paste::paste! {
            #[derive(Serialize, Deserialize, Clone, Default, Debug)]
            pub struct [<Metadata $version>] {
                $(
                    pub $field: $ty,
                )*
            }
        }
    };
}

include!(concat!(
    env!("OUT_DIR"),
    "/passphrase_metadata_generated.rs"
));
