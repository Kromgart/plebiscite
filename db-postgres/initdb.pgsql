DROP FUNCTION IF EXISTS try_login;
DROP FUNCTION IF EXISTS get_session_user;

--TMP
--DROP TABLE IF EXISTS organizations;
------

DROP TABLE IF EXISTS users_usergroups;

DROP TABLE IF EXISTS usergroups;

DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS users;

CREATE TABLE users (
    user_id        bigserial    PRIMARY KEY,
    user_name      varchar(100) NOT NULL,
    "password"     varchar(50)  NOT NULL,
    full_name      varchar(100) NOT NULL
);


CREATE TABLE sessions (
    session_id uuid           PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    bigint         NOT NULL REFERENCES users ON DELETE RESTRICT,
    expires    timestamptz(0) NOT NULL
);

--------------------------------------------------

CREATE TABLE usergroups (
    usergroup_id    bigserial PRIMARY KEY,
    title           varchar(100) NOT NULL
--    parent          bigint REFERENCES ....
);



CREATE TABLE users_usergroups (
    user_id         bigint NOT NULL REFERENCES users      ON DELETE RESTRICT,
    usergroup_id    bigint NOT NULL REFERENCES usergroups ON DELETE RESTRICT,
    PRIMARY KEY (user_id, usergroup_id)
);

--------------------------------------------------

DROP ROLE IF EXISTS pleb_reader;
CREATE ROLE pleb_reader NOLOGIN INHERIT;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO pleb_reader;


DROP ROLE IF EXISTS pleb_app;
CREATE ROLE pleb_app LOGIN PASSWORD 'aoeuAOEU';
GRANT pleb_reader TO pleb_app;

GRANT INSERT ON ALL TABLES IN SCHEMA public TO pleb_app;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO pleb_app;

GRANT DELETE ON TABLE sessions TO pleb_app;
