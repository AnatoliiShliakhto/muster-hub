use mhub_domain::constants::{QUIZ, STUDENT, SURVEY, USER, WORKSPACE};

#[test]
fn constants_match_entity_strings() {
    assert_eq!(WORKSPACE, "workspace");
    assert_eq!(USER, "user");
    assert_eq!(STUDENT, "student");
    assert_eq!(QUIZ, "quiz");
    assert_eq!(SURVEY, "survey");
}
