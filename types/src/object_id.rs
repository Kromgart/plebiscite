#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectId<T, V = i64> {
    pub value: V,
    _marker: std::marker::PhantomData<T>,
}

impl<T, V: Copy> Copy for ObjectId<T, V> { }

impl<T, V: Copy> Clone for ObjectId<T, V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "wasm")]
mod postgres {

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
            <V as FromSql>::from_sql(ty, raw).map(|v| ObjectId {
                value: v,
                _marker: std::marker::PhantomData,
            })
        }

        fn accepts(ty: &Type) -> bool {
            <V as FromSql>::accepts(ty)
        }
    }

    impl<T, V> ToSql for ObjectId<T, V> where V: ToSql, T: std::fmt::Debug {
        fn to_sql(&self, ty: &Type, out: &mut bytes::BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send + 'static>> {
            self.value.to_sql(ty, out)
        }

        fn accepts(ty: &Type) -> bool {
            <V as ToSql>::accepts(ty)
        }

        to_sql_checked!();
    }
}

/*
    use sqlx::{
        decode::Decode,
        encode::{Encode, IsNull},
        types::Type,
        Database,
    };

    use sqlx::Postgres as PG;

    impl<T, V> Type<PG> for ObjectId<T, V>
    where
        V: Type<PG>,
    {
        fn type_info() -> <PG as Database>::TypeInfo {
            <V as Type<PG>>::type_info()
        }

        fn compatible(ty: &<PG as Database>::TypeInfo) -> bool {
            <V as Type<PG>>::compatible(ty)
        }
    }

    impl<'r, T, V> Decode<'r, PG> for ObjectId<T, V>
    where
        V: Decode<'r, PG>,
    {
        fn decode(
            value: <PG as sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, Box<dyn std::error::Error + Sync + Send + 'static>> {
            <V as Decode<'r, PG>>::decode(value).map(|id| Self {
                value: id,
                _marker: std::marker::PhantomData,
            })
        }
    }

    impl<'r, T, V> Encode<'r, PG> for ObjectId<T, V>
    where
        V: Encode<'r, PG>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <PG as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
        ) -> IsNull {
            (&(self.value)).encode(buf)
        }

        fn encode(self, buf: &mut <PG as sqlx::database::HasArguments<'r>>::ArgumentBuffer) -> IsNull {
            self.value.encode(buf)
        }

        fn produces(&self) -> Option<<PG as Database>::TypeInfo> {
            self.value.produces()
        }

        fn size_hint(&self) -> usize {
            self.value.size_hint()
        }
    }
*/
