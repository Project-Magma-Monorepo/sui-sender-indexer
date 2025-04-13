-- Create required roles for Supabase
CREATE ROLE supabase_admin WITH LOGIN PASSWORD 'sui-indexer' CREATEDB CREATEROLE SUPERUSER;
CREATE ROLE authenticator WITH LOGIN PASSWORD 'sui-indexer' NOINHERIT;
CREATE ROLE anon NOINHERIT;
CREATE ROLE authenticated NOINHERIT;
CREATE ROLE service_role NOINHERIT;
CREATE ROLE supabase_auth_admin WITH LOGIN PASSWORD 'sui-indexer' NOINHERIT;
CREATE ROLE supabase_storage_admin WITH LOGIN PASSWORD 'sui-indexer' NOINHERIT;
CREATE ROLE postgres WITH LOGIN PASSWORD 'sui-indexer' SUPERUSER;

-- Create required schemas
CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS storage;
CREATE SCHEMA IF NOT EXISTS graphql_public;
CREATE SCHEMA IF NOT EXISTS _realtime;
CREATE SCHEMA IF NOT EXISTS _supabase;
CREATE SCHEMA IF NOT EXISTS _analytics;

-- Grant schema usage to roles
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role;
GRANT USAGE ON SCHEMA auth TO supabase_auth_admin, anon, authenticated, service_role;
GRANT USAGE ON SCHEMA storage TO supabase_storage_admin, anon, authenticated, service_role;
GRANT USAGE ON SCHEMA _realtime TO authenticated, service_role;

-- Create a view of your senders table if it exists
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables 
               WHERE table_schema = 'public' AND table_name = 'senders') THEN
        EXECUTE 'CREATE OR REPLACE VIEW public.supabase_senders AS SELECT * FROM public.senders';
        EXECUTE 'GRANT SELECT ON public.supabase_senders TO anon, authenticated, service_role';
    END IF;
END
$$;

-- Create a view of your blobs table if it exists
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables 
               WHERE table_schema = 'public' AND table_name = 'blobs') THEN
        EXECUTE 'CREATE OR REPLACE VIEW public.supabase_blobs AS SELECT * FROM public.blobs';
        EXECUTE 'GRANT SELECT ON public.supabase_blobs TO anon, authenticated, service_role';
    END IF;
END
$$;

-- Add extension pgcrypto which is required by Supabase Auth
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Add any other extensions that might be needed
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;