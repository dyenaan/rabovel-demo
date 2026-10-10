-- Password email addresses were not verified before assigning configured roles.
-- Remove those historical grants, including for already-active sessions (which
-- reload the principal from users). Preserve issuer accounts and their data;
-- password accounts cannot retain operational roles or elevated permissions.
UPDATE users AS u
SET roles = CASE WHEN 'issuer' = ANY(u.roles)
                 THEN ARRAY['issuer']::TEXT[] ELSE ARRAY['trader']::TEXT[] END,
    permissions = CASE WHEN 'issuer' = ANY(u.roles)
                       THEN ARRAY['view:portfolio']::TEXT[]
                       ELSE ARRAY['view:portfolio', 'trade:spot']::TEXT[] END,
    updated_at = now()
WHERE EXISTS (SELECT 1 FROM password_credentials AS p WHERE p.user_id = u.user_id);
