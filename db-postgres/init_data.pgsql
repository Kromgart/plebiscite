
DO $$
DECLARE
    admin_id users.user_id%TYPE;
BEGIN
    DELETE FROM users_usergroups;
    DELETE FROM usergroups;
    DELETE FROM users;

    INSERT INTO users (user_name, "password", full_name) 
    VALUES ('sa', '123', 'Super Admin')
    RETURNING users.user_id INTO admin_id;

    PERFORM create_assign_usergroup(admin_id, 'ACME company');
    PERFORM create_assign_usergroup(admin_id, 'Kindergarten 155, group 3');


END
$$;

