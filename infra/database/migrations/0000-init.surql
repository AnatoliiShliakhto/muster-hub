-- [CONSTANTS]
DEFINE PARAM $ACTION_CREATE VALUE "CREATE";
DEFINE PARAM $ACTION_READ   VALUE "READ";
DEFINE PARAM $ACTION_UPDATE VALUE "UPDATE";
DEFINE PARAM $ACTION_DELETE VALUE "DELETE";

-- [SESSION]
--DEFINE ACCESS users ON DATABASE TYPE RECORD WITH JWT ALGORITHM EDDSA KEY $public_key;

-- [DDL: SYSTEM]
DEFINE TABLE slice SCHEMAFULL PERMISSIONS NONE;
-- ID: <string> slug
DEFINE FIELD name           ON slice TYPE string;
DEFINE FIELD description    ON slice TYPE option<string>;

DEFINE TABLE migration SCHEMAFULL PERMISSIONS NONE;
-- ID: [<record> slice, <string> version]
DEFINE FIELD slice      ON migration TYPE record<slice> VALUE <record> id[0];
DEFINE FIELD version    ON migration TYPE string VALUE <string> id[1];
DEFINE FIELD checksum   ON migration TYPE string READONLY;

DEFINE TABLE permission SCHEMAFULL PERMISSIONS NONE;
-- ID: [<record> slice, <string> slug]
DEFINE FIELD slice          ON permission TYPE record<slice> VALUE <record> id[0];
DEFINE FIELD slug           ON permission TYPE string VALUE <string> id[1];
DEFINE FIELD name           ON permission TYPE string READONLY;

-- [DML: SYSTEM]
DEFINE FUNCTION fn::ensure_slice(
    $slice: string,
    $name: string,
    $description: option<string>
) {
    LET $s_id = type::record("slice", $slice);
    RETURN UPSERT $s_id SET name = $name, description = $description;
};

DEFINE FUNCTION fn::confirm_migration(
    $slice: string,
    $version: string,
    $checksum: string
) {
    LET $s_id = type::record("slice", $slice);
    LET $m_id = type::record("migration", [$s_id, $version]);
    LET $slice_exists = (SELECT id FROM $s_id)[0];

    IF !$slice_exists {
        THROW "Slice '" + $slice + "' not registered";
    };

    LET $existing = (SELECT checksum FROM $m_id)[0];
    IF $existing && $existing.checksum != $checksum {
        THROW "Slice '" + $slice + "' migration '" + $version + "' checksum mismatch";
    };

    CREATE $m_id SET checksum = $checksum;
};

DEFINE FUNCTION OVERWRITE fn::sync_permissions($registry: array<object>) {
    LET $unique_slices = array::distinct($registry.slice);
    LET $slice_ids = $unique_slices.map(|$s| type::record("slice", $s));
    LET $slices_count = (
        SELECT count() AS count
        FROM slice
        WHERE id IN $slice_ids
        GROUP ALL
    )[0].count;
    IF $slices_count < array::len($unique_slices) {
        THROW "One or more slices are not registered. Please register them first.";
    };
    LET $permissions = SELECT
        type::record("permission", [type::record("slice", slice), permissions]) AS id,
        <string>(slice + "." + permissions) AS name,
        permissions
    FROM $registry SPLIT permissions;
    INSERT INTO permission (SELECT id, name FROM $permissions) ON DUPLICATE KEY UPDATE id = id;
};