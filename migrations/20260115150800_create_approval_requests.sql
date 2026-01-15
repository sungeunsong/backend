-- Create extension for UUID if not exists
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE pxm_approval_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Metadata (Searchable Columns)
    title VARCHAR(255) NOT NULL,
    requester_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    
    -- Variable Data (JSONB)
    form_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    -- flow_process Structure Example:
    -- {
    --   "current_step": 1,
    --   "steps": [
    --     { "seq": 1, "approver_id": "uuid...", "status": "approved", "timestamp": "..." },
    --     { "seq": 2, "approver_id": "uuid...", "status": "pending" }
    --   ]
    -- }
    flow_process JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on commonly queried fields
CREATE INDEX idx_pxm_requests_requester_id ON pxm_approval_requests(requester_id);
CREATE INDEX idx_pxm_requests_status ON pxm_approval_requests(status);

-- GIN Index for searching specific approvers in the active steps
-- Example query: SELECT * FROM requests WHERE flow_process->'steps' @> '[{"approver_id": "...", "status": "pending"}]'
CREATE INDEX idx_pxm_requests_flow_process ON pxm_approval_requests USING gin (flow_process);
