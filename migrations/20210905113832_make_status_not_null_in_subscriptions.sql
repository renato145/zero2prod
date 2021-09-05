-- We wrap the migration in a transaction to make sure it
-- succeds or fails atomically.
BEGIN;
	-- Backfill `status` for historical entries
	UPDATE subscriptions
		SET status = 'confirmed'
		WHERE status IS NULL;
	-- Make `status` mandatory
	ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
