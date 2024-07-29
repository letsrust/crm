-- Add migration script here

CREATE TABLE user_stats(
    email varchar(128) NOT NULL PRIMARY KEY COMMENT 'user email',
    name varchar(64) NOT NULL COMMENT 'user name',
    gender char(1) DEFAULT 'U' COMMENT 'gender: M, F, U',
    created_at datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'created time',
    last_visited_at datetime COMMENT 'the last visit time',
    last_watched_at datetime COMMENT 'the last watch time',
    recent_watched text COMMENT 'recent watched id list, split by comma',
    viewed_but_not_started text COMMENT 'viewed but not started id list, split by comma',
    started_but_not_finished text COMMENT 'started but not finished id list, split by comma',
    finished text COMMENT 'finished id list, split by comma',
    last_email_notification datetime COMMENT 'the last email notfiy time',
    last_in_app_notification datetime COMMENT 'the last in-app notify time',
    last_sms_notification datetime COMMENT 'the last sms notify time',
    KEY `idx_created_at` (created_at)
) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci COMMENT 'User stats';
