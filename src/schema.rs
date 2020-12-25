table! {
    _background_tasks (id) {
        id -> Int8,
        job_type -> Text,
        is_async -> Bool,
        data -> Bytea,
        retries -> Int4,
        last_retry -> Timestamp,
        created_at -> Timestamp,
    }
}

table! {
    blocks (hash) {
        id -> Int4,
        parent_hash -> Bytea,
        hash -> Bytea,
        block_num -> Int4,
        state_root -> Bytea,
        extrinsics_root -> Bytea,
        digest -> Bytea,
        ext -> Bytea,
        spec -> Int4,
    }
}

table! {
    extrinsics (id) {
        id -> Int4,
        hash -> Bytea,
        block_num -> Int4,
        index -> Int4,
        address -> Nullable<Bytea>,
        signature -> Nullable<Bytea>,
        extra -> Nullable<Bytea>,
        fun -> Bytea,
    }
}

table! {
    metadata (version) {
        version -> Int4,
        meta -> Bytea,
    }
}

table! {
    _sqlx_migrations (version) {
        version -> Int8,
        description -> Text,
        installed_on -> Timestamptz,
        success -> Bool,
        checksum -> Bytea,
        execution_time -> Int8,
    }
}

table! {
    storage (id) {
        id -> Int4,
        block_num -> Int4,
        hash -> Bytea,
        is_full -> Bool,
        key -> Bytea,
        #[sql_name = "storage"]
        storage_data -> Nullable<Bytea>,
    }
}

joinable!(blocks -> metadata (spec));
joinable!(storage -> blocks (hash));

allow_tables_to_appear_in_same_query!(
    _background_tasks,
    blocks,
    extrinsics,
    metadata,
    _sqlx_migrations,
    storage,
);
