-- Create profiles table
CREATE TABLE profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    profile_type TEXT NOT NULL,
    location_country TEXT NOT NULL,
    location_city TEXT,
    location_currency TEXT NOT NULL,
    work_start_delay INTEGER NOT NULL,
    work_duration_limit INTEGER,
    initial_salary_usd REAL NOT NULL,
    salary_growth_rate REAL NOT NULL,
    living_cost_usd REAL NOT NULL,
    living_cost_growth REAL NOT NULL,
    tax_rate REAL NOT NULL,
    total_cost_usd REAL,
    cost_duration INTEGER,
    first_year_opportunity_cost REAL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    description TEXT
);

-- Create indexes for better query performance
CREATE INDEX idx_profiles_type ON profiles(profile_type);
CREATE INDEX idx_profiles_name ON profiles(name);
CREATE INDEX idx_profiles_created_at ON profiles(created_at);
