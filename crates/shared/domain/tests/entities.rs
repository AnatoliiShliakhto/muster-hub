use mhub_domain::entity::Entity;

#[test]
fn entity_roundtrip() {
    for (name, entity) in [
        ("workspace", Entity::Workspace),
        ("user", Entity::User),
        ("student", Entity::Student),
        ("quiz", Entity::Quiz),
        ("survey", Entity::Survey),
    ] {
        assert_eq!(entity.as_str(), name);
        assert_eq!(Entity::try_from(name).unwrap(), entity);
    }

    assert!(Entity::try_from("unknown").is_err());
}
