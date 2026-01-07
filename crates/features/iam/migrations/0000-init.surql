-- [DDL: IAM]DEFINE TABLE role SCHEMAFULL PERMISSIONS; -- for delete WHERE id != role:sa;
-- ID: <string> slug
DEFINE FIELD name           ON role TYPE string;
DEFINE FIELD description    ON role TYPE option<string>;

DEFINE TABLE role_permission SCHEMAFULL;
-- ID: [<record> role, <record> permission]
DEFINE FIELD role       ON role_permission TYPE record<role> VALUE <record> id[0];
DEFINE FIELD permission ON role_permission TYPE record<permission> VALUE <record> id[1];
DEFINE FIELD actions    ON role_permission TYPE array<string> DEFAULT [];

DEFINE TABLE assignment SCHEMAFULL;
-- ID: [<record> organization <record> user, <record> role]
DEFINE FIELD org        ON assignment TYPE record<organization> VALUE <record> id[0];
DEFINE FIELD user       ON assignment TYPE record<user> VALUE <record> id[1];
DEFINE FIELD role       ON assignment TYPE record<role> VALUE <record> id[2];
DEFINE INDEX idx_user   ON assignment FIELDS user;

-- [DDL: ACCESS CACHE]
DEFINE TABLE access_cache SCHEMAFULL PERMISSIONS NONE;
-- ID: [<record> user, <record> organization]
DEFINE FIELD user       ON access_cache TYPE record<user> VALUE <record> id[0];
DEFINE FIELD org        ON access_cache TYPE record<organization> VALUE <record> id[1];
-- policy: { "permission_name": ['CREATE', 'READ', 'UPDATE', 'DELETE'] }
DEFINE FIELD policy     ON access_cache TYPE object DEFAULT {};
DEFINE FIELD policy.*   ON access_cache TYPE array<string>;
DEFINE FIELD updated_at ON access_cache TYPE datetime DEFAULT time::now();

-- [DML: ACCESS CACHE]
DEFINE FUNCTION fn::sync_user_access_cache($user: record<user>) {
    LET $data = (
        SELECT
            (SELECT VALUE id FROM organization WHERE id = $parent.org OR chain CONTAINS $parent.org) AS org_list,
            (SELECT permission.name as permission, actions FROM role_permission WHERE role = $parent.role) AS perm_list
        FROM assignment WHERE user = $user
    );

    LET $final_cache = (
        SELECT
            org_list AS org,
            object::from_entries(
                array::filter(
                    array::map(
                        array::group(perm_list),
                        |$group| {
                            LET $p_name = $group.permission;
                            RETURN IF $p_name != NONE {
                                [ $p_name, array::distinct(array::flatten($group.actions)) ]
                            } ELSE { NONE };
                        }
                    ),
                    |$v| $v != NONE
                )
            ) AS policy
        FROM (
            SELECT org_list, perm_list FROM $data WHERE org_list != NONE SPLIT ON org_list
        )
        GROUP BY org
    );

    DELETE access_cache WHERE id INSIDE $user;

    FOR $row IN $final_cache {
        LET $cache_id = type::record('access_cache', [$user, $row.org]);
        UPSERT $cache_id SET policy = $row.policy;
    };

    RETURN true;
};

-- [EVENTS]
DEFINE EVENT IF NOT EXISTS audit ON role WHEN $before != $after THEN {
    fn::audit_logger($before, $after)
};

DEFINE EVENT IF NOT EXISTS audit ON role_permission WHEN $before != $after THEN {
    fn::audit_logger($before, $after)
};

DEFINE EVENT IF NOT EXISTS audit ON assignment WHEN $before != $after THEN {
    fn::audit_logger($before, $after)
};

DEFINE EVENT sync_on_assignment ON TABLE assignment WHEN $event = 'CREATE' OR $event = 'DELETE' THEN {
    fn::sync_user_access_cache($after.user OR $before.user);
};

DEFINE EVENT sync_on_role_change ON TABLE role_permission WHEN $event = 'UPDATE' THEN {
    LET $affected_users = (SELECT VALUE user FROM assignment WHERE role = $after.role);
    -- It's a heavy op: would be refactored in the future into a cron job or eg.
    FOR $user in $affected_users {
        fn::sync_user_access_cache($user);
    };
};