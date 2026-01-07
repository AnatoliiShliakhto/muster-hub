-- [DDL: ORGANIZATION]
DEFINE TABLE organization SCHEMAFULL;
-- ID: <string> slug
DEFINE FIELD name       ON organization TYPE string;
DEFINE FIELD parent     ON organization TYPE option<record<organization>>;
DEFINE FIELD chain      ON organization TYPE array<record<organization>> DEFAULT [id];
DEFINE INDEX idx_parent ON organization FIELDS parent;
DEFINE INDEX idx_chain  ON organization FIELDS chain;

-- [EVENTS]
DEFINE EVENT IF NOT EXISTS audit ON organization WHEN $before != $after THEN {
    fn::audit_logger($before, $after)
};

DEFINE EVENT hierarchy_sync ON organization WHEN $before.parent != $after.parent THEN {
    LET $new_chain = IF $after.parent == NONE {
        [ $after.id ]
    } ELSE {
        LET $p_chain = (SELECT VALUE chain FROM $after.parent)[0];
        array::add($p_chain, $after.id)
    };

    UPDATE $after.id SET chain = $new_chain;

    LET $descendants = (SELECT id, parent FROM organization WHERE chain CONTAINS $after.id AND id != $after.id);

    FOR $child in $descendants {
        LET $child_parent_chain = (SELECT VALUE chain FROM $child.parent)[0];
        UPDATE $child.id SET chain = array::add($child_parent_chain, $child.id);
    };
};
