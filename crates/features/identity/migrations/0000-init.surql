-- [DDL: USERS & POLICIES]
--DEFINE SEQUENCE user BATCH 1000 START 1000 TIMEOUT 5s;
DEFINE TABLE user SCHEMAFULL; -- PERMISSIONS for delete WHERE id.id() >= 1000;
-- ID: <int> sequence::nextval("user") -- I'm thought on that?!
DEFINE FIELD login      ON user TYPE string;
DEFINE FIELD password   ON user TYPE option<string>;
DEFINE FIELD status     ON user TYPE string DEFAULT 'ACTIVE' ASSERT $value in ['ACTIVE', 'INACTIVE', 'ARCHIVED'];
DEFINE INDEX idx_login  ON user FIELDS login UNIQUE;
DEFINE INDEX idx_status ON user FIELDS status;

-- [EVENTS]
DEFINE EVENT IF NOT EXISTS audit ON user WHEN $before != $after THEN {
    fn::audit_logger($before, $after)
};
