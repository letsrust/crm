-- Add migration script here
alter table user_stats add index `idx_last_visited_at` (last_visited_at);
alter table user_stats add index `idx_last_watched_at` (last_watched_at);
