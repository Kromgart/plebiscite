use std::marker::PhantomData;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq)]
pub struct ObjectId<T, V = i64> {
    pub value: V,
    _marker: PhantomData<T>,
}

impl<T, V> ObjectId<T, V> {
    pub fn new(value: V) -> Self {
        Self {
            value,
            _marker: PhantomData
        }
    }
}

impl<T, V: Copy> Copy for ObjectId<T, V> { }

impl<T, V: Copy> Clone for ObjectId<T, V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            _marker: PhantomData,
        }
    }
}

impl<T, V: Serialize> serde::Serialize for ObjectId<T, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
    where
        S: serde::Serializer
    {
        self.value.serialize(serializer)
    }
}

impl<'a, T, V: Deserialize<'a>> serde::Deserialize<'a> for ObjectId<T, V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> 
    where 
        D: serde::Deserializer<'a>
    {
        V::deserialize(deserializer)
            .map(|v| Self {
                value: v,
                _marker: PhantomData,
            })
    }
}

#[cfg(feature = "wasm")]
mod wasm {

    use super::ObjectId;
    use std::hash::{Hash, Hasher};

    impl<T, V: Hash> Hash for ObjectId<T, V> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.value.hash(state)
        }
    }
}

#[cfg(feature = "postgres")]
mod postgres {

    use super::ObjectId;
    use std::error::Error;
    use postgres_types::{FromSql, ToSql, Type, IsNull, to_sql_checked};

    impl<'a, T, V> FromSql<'a> for ObjectId<T, V>
    where
        V: FromSql<'a>,
    {
        fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
            V::from_sql(ty, raw).map(|v| ObjectId {
                value: v,
                _marker: std::marker::PhantomData,
            })
        }

        fn accepts(ty: &Type) -> bool {
            V::accepts(ty)
        }
    }

    impl<T, V> ToSql for ObjectId<T, V> where V: ToSql, T: std::fmt::Debug {
        fn to_sql(&self, ty: &Type, out: &mut bytes::BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send + 'static>> {
            self.value.to_sql(ty, out)
        }

        fn accepts(ty: &Type) -> bool {
            V::accepts(ty)
        }

        to_sql_checked!();
    }
}
