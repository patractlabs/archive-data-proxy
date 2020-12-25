///
/// 查询Archive 的Postgres 数据库数据，但调用方要根据返回结果自己组装才能符合RPC返回标准格式
///

#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::schema::storage::dsl::storage;
//use diesel::expression::dsl::*;
use diesel::dsl::sql;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use models::*;
use schema::blocks::dsl::*;
use schema::storage::dsl::{hash as HashOfStorage, key};

#[allow(unused)]
pub struct ArchiveDataProxy {
    url: String,
    connection: PgConnection,
}
impl ArchiveDataProxy {
    const PAGE_SIZE: i64 = 10000;
    pub fn new(u: String) -> ArchiveDataProxy {
        let con = PgConnection::establish(&u.clone()).expect(&format!("Error connecting to {}", u));
        Self {
            url: u.clone(),
            connection: con,
        }
    }
    // 支持RPC chain_getBlockHash 如果客户端传过来的是hex的高度，现在外层转换成number再调用这里的接口
    // 不支持RPC chain_getBlock 因为archive不支持SignedBlock的justification
    pub fn query_block_header_by_number(&mut self, numbers: Vec<i32>) -> Vec<Block> {
        let data = blocks
            .filter(block_num.eq_any(numbers))
            .limit(Self::PAGE_SIZE)
            .load::<Block>(&self.connection)
            .expect("Error loading block");

        data
    }
    // 支持RPC chain_getHeader
    pub fn query_block_header_by_hash(&mut self, hashs: Vec<Vec<u8>>) -> Vec<Block> {
        let data = blocks
            .filter(hash.eq_any(hashs))
            .limit(Self::PAGE_SIZE)
            .load::<Block>(&self.connection)
            .expect("Error loading block");

        data
    }
    // 支持RPC　state_getStorage
    // todo 支持翻页
    pub fn query_storage_by_key_and_hash(
        &mut self,
        k: Vec<u8>,
        block_hash: Option<Vec<u8>>,
    ) -> Vec<Storage> {
        match block_hash {
            Some(h) => storage
                .filter(key.eq(k))
                .filter(HashOfStorage.eq(h))
                .limit(Self::PAGE_SIZE)
                .load::<Storage>(&self.connection)
                .unwrap_or(Vec::new()),
            None => storage
                .filter(key.eq(k))
                .limit(Self::PAGE_SIZE)
                .load::<Storage>(&self.connection)
                .unwrap_or(Vec::new()),
        }
    }

    // 支持RPC state_queryStorage
    pub fn query_storage_by_keys_and_from_to(
        &mut self,
        keys: Vec<Vec<u8>>,
        from_hash: Vec<u8>,
        to_hash: Option<Vec<u8>>,
    ) -> Vec<Storage> {
        let start_blocks = self.query_block_header_by_hash([from_hash].to_vec());
        if start_blocks.len() < 1 {
            return Vec::new();
        }

        let start_block_num = start_blocks[0].block_num;
        let end_block_num = to_hash.map_or(start_block_num, |to| {
            let end_blocks = self.query_block_header_by_hash([to].to_vec());
            if end_blocks.len() < 1 {
                start_block_num
            } else {
                end_blocks[0].block_num
            }
        });

        let between = format!(
            "block_num >={} and block_num <={}",
            start_block_num, end_block_num
        );

        storage
            .filter(key.eq_any(keys))
            .filter(sql(between.as_ref()))
            //.filter(block_num.between(start_block_num,end_block_num))
            .limit(Self::PAGE_SIZE)
            .load::<Storage>(&self.connection)
            .unwrap_or(Vec::new())
    }
    // 支持RPC state_queryStorageAt
    pub fn query_storage_by_keys_at_hash(
        &mut self,
        keys: Vec<Vec<u8>>,
        at: Option<Vec<u8>>,
    ) -> Vec<Storage> {
        match at {
            Some(h) => storage
                .filter(key.eq_any(keys))
                .filter(HashOfStorage.eq(h))
                .limit(Self::PAGE_SIZE)
                .load::<Storage>(&self.connection)
                .unwrap_or(Vec::new()),
            None => storage
                .filter(key.eq_any(keys))
                .limit(Self::PAGE_SIZE)
                .load::<Storage>(&self.connection)
                .unwrap_or(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Block, Storage};
    use crate::ArchiveDataProxy;

    fn print_storage(storages: Vec<Storage>) {
        for s in storages {
            println!("id:{}", s.id);
            println!("block_num:{}", s.block_num);
            println!("hash:{:?}", hex::encode(s.hash));
            println!("is_full:{:?}", s.is_full);
            println!("key:{:?}", hex::encode(s.key));
            println!("data:{:?}", hex::encode(s.data.unwrap_or(Vec::new())));
            println!("----------\n");
        }
    }
    fn print_headers(headers: Vec<Block>) {
        for header in headers {
            println!("id:{}", header.id);
            println!("parent_hash:{:?}", hex::encode(header.parent_hash));
            println!("hash:{:?}", hex::encode(header.hash));
            println!("block_num:{}", header.block_num);
            println!("state_root:{:?}", hex::encode(header.state_root));
            println!("extrinsics_root:{:?}", hex::encode(header.extrinsics_root));
            println!("digest:{:?}", hex::encode(header.digest));
            println!("ext:{:?}", hex::encode(header.ext));
            println!("spec:{}", header.spec);
            println!("----------\n");
        }
    }
    #[test]
    fn query_block_header_by_number() {
        let mut proxy = ArchiveDataProxy::new(String::from(
            "postgres://postgres:123@localhost/polkadot_db",
        ));
        let numbers = vec![2, 3];
        let data = proxy.query_block_header_by_number(numbers);
        print_headers(data.clone());
        assert_eq!(2, data.len());
    }
    #[test]
    fn query_block_header_by_hash() {
        let mut proxy = ArchiveDataProxy::new(String::from(
            "postgres://postgres:123@localhost/polkadot_db",
        ));
        let h2 = hex::decode("409d0bfe677594d7558101d574633d5808a6fc373cbd964ef236f00941f290ee")
            .unwrap();
        let h3 = hex::decode("5b940c7fc0a1c5a58e4d80c5091dd003303b8f18e90a989f010c1be6f392bed1")
            .unwrap();

        let hashs = vec![h2, h3];
        let data = proxy.query_block_header_by_hash(hashs);
        print_headers(data.clone());
        assert_eq!(2, data.len());
    }
    #[test]
    fn query_storage_by_key_and_hash() {
        let mut proxy = ArchiveDataProxy::new(String::from(
            "postgres://postgres:123@localhost/polkadot_db",
        ));
        let key = hex::decode("3fba98689ebed1138735e0e7a5a790abb984cfb497221deefcefb70073dcaac1")
            .unwrap();
        let h2 = hex::decode("409d0bfe677594d7558101d574633d5808a6fc373cbd964ef236f00941f290ee")
            .unwrap();
        let data = proxy.query_storage_by_key_and_hash(key, Some(h2));
        print_storage(data.clone());
        assert_eq!(2, data[0].block_num);
    }

    #[test]
    fn query_storage_by_keys_and_from_to() {
        let mut proxy = ArchiveDataProxy::new(String::from(
            "postgres://postgres:123@localhost/polkadot_db",
        ));
        let key1 = hex::decode("3fba98689ebed1138735e0e7a5a790abb984cfb497221deefcefb70073dcaac1")
            .unwrap();
        let key2 = hex::decode("3fba98689ebed1138735e0e7a5a790ab21a5051453bd3ae7ed269190f4653f3b")
            .unwrap();
        let h2 = hex::decode("409d0bfe677594d7558101d574633d5808a6fc373cbd964ef236f00941f290ee")
            .unwrap();
        let h4 = hex::decode("d380bee22de487a707cbda65dd9d4e2188f736908c42cf390c8919d4f7fc547c")
            .unwrap();
        let data = proxy.query_storage_by_keys_and_from_to(vec![key1, key2], h2, Some(h4));
        print_storage(data.clone());
        assert_eq!(6, data.len());
    }

    #[test]
    fn query_storage_by_keys_at_hash() {
        let mut proxy = ArchiveDataProxy::new(String::from(
            "postgres://postgres:123@localhost/polkadot_db",
        ));
        let key1 = hex::decode("3fba98689ebed1138735e0e7a5a790abb984cfb497221deefcefb70073dcaac1")
            .unwrap();
        let key2 = hex::decode("3fba98689ebed1138735e0e7a5a790ab21a5051453bd3ae7ed269190f4653f3b")
            .unwrap();
        let h2 = hex::decode("409d0bfe677594d7558101d574633d5808a6fc373cbd964ef236f00941f290ee")
            .unwrap();

        let data = proxy.query_storage_by_keys_at_hash(vec![key1, key2], Some(h2));
        print_storage(data.clone());
        assert_eq!(2, data.len());
    }
}
