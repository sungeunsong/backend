CREATE TABLE approval_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    approval_id UUID NOT NULL REFERENCES pxm_approval_requests(id) ON DELETE CASCADE,
    actor_id UUID NOT NULL,
    action_type VARCHAR(50) NOT NULL, -- 'CREATED', 'APPROVED', 'REJECTED', 'COMMENT'
    content TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_approval_logs_approval_id ON approval_logs(approval_id);
