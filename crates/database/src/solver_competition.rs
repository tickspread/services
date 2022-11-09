use crate::{auction::AuctionId, TransactionHash};
use sqlx::{types::JsonValue, PgConnection};

pub async fn save(
    ex: &mut PgConnection,
    id: AuctionId,
    data: &JsonValue,
    tx_hash: Option<&TransactionHash>,
) -> Result<(), sqlx::Error> {
    const QUERY: &str = r#"
INSERT INTO solver_competitions (id, json, tx_hash)
VALUES ($1, $2, $3)
ON CONFLICT (id) DO UPDATE
SET json = EXCLUDED.json, tx_hash = EXCLUDED.tx_hash
    ;"#;
    sqlx::query(QUERY)
        .bind(id)
        .bind(data)
        .bind(tx_hash)
        .execute(ex)
        .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct LoadById {
    pub json: JsonValue,
    pub tx_hash: Option<TransactionHash>,
}

pub async fn load_by_id(
    ex: &mut PgConnection,
    id: AuctionId,
) -> Result<Option<LoadById>, sqlx::Error> {
    const QUERY: &str = r#"
SELECT json, tx_hash
FROM solver_competitions
WHERE id = $1
    ;"#;
    sqlx::query_as(QUERY).bind(id).fetch_optional(ex).await
}

#[derive(sqlx::FromRow)]
pub struct LoadByTxHash {
    pub json: JsonValue,
    pub id: AuctionId,
}

pub async fn load_by_tx_hash(
    ex: &mut PgConnection,
    tx_hash: &TransactionHash,
) -> Result<Option<LoadByTxHash>, sqlx::Error> {
    const QUERY: &str = r#"
SELECT json, id
FROM solver_competitions
WHERE tx_hash = $1
    ;"#;
    sqlx::query_as(QUERY).bind(tx_hash).fetch_optional(ex).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::byte_array::ByteArray;
    use sqlx::Connection;

    #[tokio::test]
    #[ignore]
    async fn postgres_roundtrip() {
        let mut db = PgConnection::connect("postgresql://").await.unwrap();
        let mut db = db.begin().await.unwrap();
        crate::clear_DANGER_(&mut db).await.unwrap();

        let value = JsonValue::Bool(true);
        save(&mut db, 0, &value, None).await.unwrap();
        let value_ = load_by_id(&mut db, 0).await.unwrap().unwrap();
        assert_eq!(value, value_.json);
        assert!(value_.tx_hash.is_none());

        assert!(load_by_id(&mut db, 1).await.unwrap().is_none());

        let value = JsonValue::String("a".to_string());
        let tx_hash = ByteArray([0x01; 32]);
        save(&mut db, 0, &value, Some(&tx_hash)).await.unwrap();
        let value_ = load_by_id(&mut db, 0).await.unwrap().unwrap();
        assert_eq!(value, value_.json);
        assert_eq!(value_.tx_hash, Some(tx_hash));
    }

    #[tokio::test]
    #[ignore]
    async fn postgres_by_hash() {
        let mut db = PgConnection::connect("postgresql://").await.unwrap();
        let mut db = db.begin().await.unwrap();
        crate::clear_DANGER_(&mut db).await.unwrap();

        let value = JsonValue::Bool(true);
        let hash = ByteArray([1u8; 32]);
        save(&mut db, 0, &value, Some(&hash)).await.unwrap();

        let value_by_id = load_by_id(&mut db, 0).await.unwrap().unwrap();
        let value_by_hash = load_by_tx_hash(&mut db, &hash).await.unwrap().unwrap();
        assert_eq!(value, value_by_id.json);
        assert_eq!(value_by_id.tx_hash, Some(hash));
        assert_eq!(value, value_by_hash.json);
        assert_eq!(value_by_hash.id, 0);

        let not_found = load_by_tx_hash(&mut db, &ByteArray([2u8; 32]))
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn postgres_can_overwrite() {
        let mut db = PgConnection::connect("postgresql://").await.unwrap();
        let mut db = db.begin().await.unwrap();
        crate::clear_DANGER_(&mut db).await.unwrap();

        let value = JsonValue::Bool(true);
        save(&mut db, 0, &value, None).await.unwrap();
        let value_ = load_by_id(&mut db, 0).await.unwrap().unwrap();
        assert_eq!(value, value_.json);
        assert!(value_.tx_hash.is_none());

        // overwrite id
        let value = JsonValue::Bool(false);
        let hash = ByteArray([1u8; 32]);
        save(&mut db, 0, &value, Some(&hash)).await.unwrap();
        let value_ = load_by_id(&mut db, 0).await.unwrap().unwrap();
        assert_eq!(value, value_.json);
        assert_eq!(value_.tx_hash, Some(hash));
    }
}
