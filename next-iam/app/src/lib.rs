#[cfg(test)]
mod tests {
    use domain::user::User;
    use uuid::Uuid;

    #[test]
    fn new_user() {
        let u = User {
            id: Uuid::new_v4(),
            name: "alice".into(),
            email: "alice@example.com".into(),
        };
        assert_eq!(u.name, "alice");
    }
}
