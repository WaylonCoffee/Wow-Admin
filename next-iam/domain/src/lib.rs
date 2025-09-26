pub mod user {
    use serde::{Serialize, Deserialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct User {
        pub id: Uuid,
        pub name: String,
        pub email: String
    }
}