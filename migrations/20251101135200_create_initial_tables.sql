-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    IF NOT EXISTS users (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
        username VARCHAR(255) NOT NULL UNIQUE,
        business_name VARCHAR(255) NOT NULL,
        email VARCHAR(255) NOT NULL UNIQUE,
        location VARCHAR(255),
        phone_number VARCHAR(20),
        cover_image_url VARCHAR(512),
        profile_image_url VARCHAR(512),
        description TEXT,
        is_verified BOOLEAN DEFAULT FALSE,
        google_is_connected BOOLEAN DEFAULT FALSE,
        phone_number_is_whatsapp BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMPTZ DEFAULT NOW (),
        updated_at TIMESTAMPTZ DEFAULT NOW (),
        last_login TIMESTAMPTZ DEFAULT NOW ()
    );

CREATE TABLE
    IF NOT EXISTS auth (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
        user_id UUID NOT NULL REFERENCES users (id),
        google_id VARCHAR(255) NOT NULL,
        refresh_token TEXT,
        created_at TIMESTAMPTZ DEFAULT NOW (),
        updated_at TIMESTAMPTZ DEFAULT NOW (),
        --
        -- Ensures a user can only have one auth entry
        CONSTRAINT auth_user_id_key UNIQUE (user_id),
        -- Ensures a Google ID can only be used once
        CONSTRAINT auth_google_id_key UNIQUE (google_id)
    );

CREATE TABLE
    IF NOT EXISTS services (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
        user_id UUID NOT NULL REFERENCES users (id),
        service_name VARCHAR(255) NOT NULL,
        description TEXT,
        price DECIMAL(10, 2),
        duration_minutes INT,
        category VARCHAR(100),
        created_at TIMESTAMPTZ DEFAULT NOW (),
        updated_at TIMESTAMPTZ DEFAULT NOW ()
    );

CREATE TABLE
    IF NOT EXISTS appointments (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
        service_id UUID NOT NULL REFERENCES services (id),
        business_id UUID NOT NULL REFERENCES users (id),
        customer_name VARCHAR(255) NOT NULL,
        customer_email VARCHAR(255),
        customer_phone VARCHAR(20),
        appointment_time TIMESTAMPTZ NOT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW (),
        updated_at TIMESTAMPTZ DEFAULT NOW ()
    );