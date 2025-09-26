pub mod user_repo {
    use sqlx::{PgPool};
    use uuid::Uuid;
    use anyhow::Result;
    use domain::user::User;

    pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query_as!(
            User,
            r#"
            SELECT id, name, email
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }
}