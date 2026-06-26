-- Enable pgvector extension for face embedding storage
CREATE EXTENSION IF NOT EXISTS vector WITH SCHEMA public;
ALTER EXTENSION vector SET SCHEMA public;

-- Shared layer: face detection results cache (no user_id, objective data)
CREATE TABLE image_face_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    image_hash TEXT NOT NULL,
    source_app TEXT NOT NULL,
    source_id TEXT NOT NULL,
    face_index INT NOT NULL DEFAULT 0,
    embedding vector(512) NOT NULL,
    bbox JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(image_hash, face_index)
);

CREATE INDEX idx_image_face_cache_source ON image_face_cache(source_app, source_id);
CREATE INDEX idx_image_face_cache_embedding ON image_face_cache USING hnsw (embedding vector_cosine_ops);

-- User layer: persons (user-scoped)
CREATE TABLE persons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name TEXT,
    avatar_url TEXT,
    face_count INT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_persons_user_id ON persons(user_id);

-- User layer: person-face associations (user-scoped)
CREATE TABLE person_faces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    person_id UUID NOT NULL REFERENCES persons(id) ON DELETE CASCADE,
    face_cache_id UUID NOT NULL REFERENCES image_face_cache(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, face_cache_id)
);

CREATE INDEX idx_person_faces_user_person ON person_faces(user_id, person_id);
CREATE INDEX idx_person_faces_cache ON person_faces(face_cache_id);

-- User layer: person-media associations (user-scoped)
CREATE TABLE person_media (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    person_id UUID NOT NULL REFERENCES persons(id) ON DELETE CASCADE,
    source_app TEXT NOT NULL,
    source_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, person_id, source_app, source_id)
);

CREATE INDEX idx_person_media_user_person ON person_media(user_id, person_id);
CREATE INDEX idx_person_media_source ON person_media(user_id, source_app, source_id);
