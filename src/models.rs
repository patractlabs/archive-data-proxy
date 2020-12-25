#[allow(unused)]
#[derive(Queryable, Clone)]
pub struct Block {
    pub id: i32,
    pub parent_hash: Vec<u8>,
    pub hash: Vec<u8>,
    pub block_num: i32,
    pub state_root: Vec<u8>,
    pub extrinsics_root: Vec<u8>,
    pub digest: Vec<u8>,
    pub ext: Vec<u8>,
    pub spec: i32,
}
#[allow(unused)]
#[derive(Queryable, Clone)]
pub struct Storage {
    pub id: i32,
    pub block_num: i32,
    pub hash: Vec<u8>,
    pub is_full: bool,
    pub key: Vec<u8>,
    pub data: Option<Vec<u8>>,
}
