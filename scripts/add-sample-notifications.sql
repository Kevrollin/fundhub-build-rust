-- Add sample notifications for testing
-- Replace 'user_id_here' with actual user IDs from your database

-- Get a sample user ID (replace with actual user ID)
-- INSERT INTO notifications (user_id, notification_type, title, message, metadata, is_read, created_at)
-- VALUES 
--   (
--     (SELECT id FROM users LIMIT 1),
--     'donation',
--     'New Donation Received!',
--     'You received a donation of $50 for your "AI Learning Platform" project.',
--     '{"amount": 50, "project_id": "sample-project-id"}',
--     false,
--     NOW()
--   ),
--   (
--     (SELECT id FROM users LIMIT 1),
--     'project',
--     'Project Approved',
--     'Your project "Blockchain Education App" has been approved and is now live!',
--     '{"project_id": "blockchain-education-app"}',
--     false,
--     NOW() - INTERVAL '1 hour'
--   ),
--   (
--     (SELECT id FROM users LIMIT 1),
--     'verification',
--     'Student Verification Complete',
--     'Your student verification has been approved. You can now create projects and receive funding.',
--     '{}',
--     true,
--     NOW() - INTERVAL '2 hours'
--   ),
--   (
--     (SELECT id FROM users LIMIT 1),
--     'system',
--     'Welcome to FundHub!',
--     'Welcome to FundHub! Start by exploring projects or creating your own.',
--     '{}',
--     true,
--     NOW() - INTERVAL '1 day'
--   ),
--   (
--     (SELECT id FROM users LIMIT 1),
--     'campaign',
--     'New Campaign Available',
--     'A new funding campaign "Tech Innovation Challenge" is now available for applications.',
--     '{"campaign_id": "tech-innovation-challenge"}',
--     false,
--     NOW() - INTERVAL '30 minutes'
--   );

-- Alternative: Insert for all users
INSERT INTO notifications (user_id, notification_type, title, message, metadata, is_read, created_at)
SELECT 
    u.id,
    'system',
    'Welcome to FundHub!',
    'Welcome to FundHub! Start by exploring projects or creating your own.',
    '{}',
    true,
    NOW() - INTERVAL '1 day'
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM notifications n 
    WHERE n.user_id = u.id 
    AND n.notification_type = 'system' 
    AND n.title = 'Welcome to FundHub!'
);
