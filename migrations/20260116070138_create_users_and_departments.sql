-- Drop tables if they exist to force clean slate for Enterprise Schema
DROP TABLE IF EXISTS departments CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- 1. Departments Table (Hierarchy)
CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(50) UNIQUE NOT NULL, -- e.g. HR01, DEV02
    parent_id UUID REFERENCES departments(id), -- Hierarchical structure
    manager_id UUID, -- Will define FK later to avoid circular dependency initially or handle carefully
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 2. Users Table (Enterprise Grade)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Identity
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL, -- Argon2 hash
    
    -- Profile
    full_name VARCHAR(100) NOT NULL,
    position VARCHAR(100), -- Job Title e.g. "Senior Engineer"
    
    -- Organization
    department_id UUID REFERENCES departments(id) ON DELETE SET NULL,
    
    -- State
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE', -- ACTIVE, INACTIVE, SUSPENDED
    last_login_at TIMESTAMP WITH TIME ZONE,
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 3. Circular Dependency Resolution (Department Manager)
ALTER TABLE departments 
ADD CONSTRAINT fk_departments_manager 
FOREIGN KEY (manager_id) REFERENCES users(id) ON DELETE SET NULL;

-- 4. Indexes for Performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_department ON users(department_id);
CREATE INDEX idx_departments_parent ON departments(parent_id);

-- 5. Seed Initial Admin User & Root Department (Optional but good for quick start)
-- Password for 'admin123' (Argon2 hash example placeholder, handled in app logic usually)
-- For now, let's keep it clean. We will create a Seeder script or endpoint.
