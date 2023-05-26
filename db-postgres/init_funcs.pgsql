DROP FUNCTION IF EXISTS add_session;
CREATE FUNCTION add_session(
    __user_id sessions.user_id%TYPE
) RETURNS sessions.session_id%TYPE
AS $$
DECLARE
    __session_id sessions.session_id%TYPE;
BEGIN
    DELETE FROM sessions WHERE user_id = __user_id AND current_timestamp >= expires;

    INSERT INTO sessions (user_id, expires)
    VALUES (__user_id, current_timestamp + interval '30 minutes')
    RETURNING session_id INTO __session_id;

    RETURN __session_id;
END
$$ LANGUAGE plpgsql;



DROP FUNCTION IF EXISTS try_login;
CREATE FUNCTION try_login(
    __user_name users.user_name%TYPE,
    __password users."password"%TYPE
) RETURNS sessions.session_id%TYPE 
AS $$
DECLARE
    __user_id users.user_id%TYPE;
    __session_id sessions.session_id%TYPE := NULL;
BEGIN
    SELECT user_id INTO __user_id FROM users WHERE user_name = __user_name AND "password" = __password LIMIT 1;
    IF __user_id IS NOT NULL THEN
        SELECT add_session FROM add_session(__user_id) INTO __session_id;
    END IF;

    RETURN __session_id;
END
$$ LANGUAGE plpgsql;



DROP FUNCTION IF EXISTS try_register_login;
CREATE FUNCTION try_register_login(
    __user_name users.user_name%TYPE,
    __password users."password"%TYPE
) RETURNS sessions.session_id%TYPE 
AS $$
DECLARE
    __user_id users.user_id%TYPE;
    __session_id sessions.session_id%TYPE := NULL;
BEGIN
    SELECT user_id INTO __user_id FROM users WHERE user_name = __user_name LIMIT 1;
    IF __user_id IS NULL THEN
        WITH usr AS (
            INSERT INTO users (user_name, "password", full_name)
            VALUES (__user_name, __password, '(full name)')
            RETURNING user_id
        )
        SELECT usr.user_id FROM usr INTO __user_id;

        SELECT add_session FROM add_session(__user_id) INTO __session_id;
    END IF;

    RETURN __session_id;
END
$$ LANGUAGE plpgsql;



DROP FUNCTION IF EXISTS get_session_user;
CREATE FUNCTION get_session_user(
    __session_id sessions.session_id%TYPE
) RETURNS TABLE(
    user_id users.user_id%TYPE,
    user_name users.user_name%TYPE,
    full_name users.full_name%TYPE
) AS $$
BEGIN
    RETURN QUERY SELECT u.user_id, u.user_name, u.full_name 
    FROM sessions ss INNER JOIN users u ON ss.user_id = u.user_id
    WHERE ss.session_id = __session_id AND current_timestamp < ss.expires;

    RETURN;
END
$$ LANGUAGE plpgsql;



DROP FUNCTION IF EXISTS create_assign_usergroup;
CREATE FUNCTION create_assign_usergroup(
    __user_id users.user_id%TYPE,
    __title usergroups.title%TYPE
) RETURNS usergroups.usergroup_id%TYPE
AS $$
DECLARE
    __group_id usergroups.usergroup_id%TYPE;
BEGIN
    INSERT INTO usergroups (title) VALUES (__title) 
    RETURNING usergroups.usergroup_id INTO __group_id;

    INSERT INTO users_usergroups (user_id, usergroup_id)
    VALUES (__user_id, __group_id);

    RETURN __group_id;
END 
$$ LANGUAGE plpgsql;



DROP FUNCTION IF EXISTS get_assigned_usergroups;
CREATE FUNCTION get_assigned_usergroups(
    __user_id users.user_id%TYPE
) RETURNS TABLE(
    usergroup_id usergroups.usergroup_id%TYPE,
    title usergroups.title%TYPE
) AS $$
BEGIN
    RETURN QUERY SELECT g.usergroup_id, g.title 
    FROM usergroups g LEFT JOIN users_usergroups ug ON ug.usergroup_id = g.usergroup_id
    WHERE ug.user_id = __user_id;

    RETURN;
END 
$$ LANGUAGE plpgsql;


