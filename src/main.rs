#[macro_use]
extern crate dotenv;

use dotenv::dotenv;
use iroh::SecretKey;
use iroh::{Endpoint, endpoint::presets, protocol::Router};

use iroh_blobs::{
    ALPN as BLOBS_ALPN, BlobsProtocol, Hash, store::fs::FsStore, store::mem::MemStore,
};
use iroh_docs::Capability;
use iroh_docs::{
    ALPN as DOCS_ALPN, AuthorId, DocTicket,
    api::{Doc, protocol::ShareMode},
    engine::LiveEvent,
    protocol::Docs,
    store::Query,
    sync::Entry,
};
use iroh_gossip::{ALPN as GOSSIP_ALPN, net::Gossip};
use n0_future::StreamExt;

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::PathBuf;
const TABLE: TableDefinition<&str, &str> = TableDefinition::new("blobs");
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // create an iroh endpoint that includes the standard address lookup mechanisms
    // we've built at number0
    let key = dotenv::var("KEY").unwrap();
    let secret_key = SecretKey::try_from(&key.as_bytes()[0..32])?;
    let path_raw = dotenv::var("FSPATH").unwrap();
    println!("We are loading from {}", path_raw.clone());

    let path = PathBuf::from(path_raw);

    let endpoint = iroh::Endpoint::builder(iroh::endpoint::presets::N0)
        .secret_key(secret_key)
        .bind()
        .await?;

    // // build the blobs protocol
    eprintln!("loading blobs");
    let blobs: FsStore = FsStore::load(&path).await?;
    eprintln!("build endpoint");
    // build the gossip proto   ol
    let gossip: Gossip = Gossip::builder().spawn(endpoint.clone());
    eprintln!("loading docs");
    // build the docs protocol
    let docs = Docs::persistent(path)
        .spawn(endpoint.clone(), (*blobs).clone(), gossip.clone())
        .await?;
    eprintln!("docs spawned");
    let author = docs.author_create().await?;
    eprintln!("author created");
    // create a router builder, we will add the
    // protocols to this builder and then spawn
    // the router
    let builder = Router::builder(endpoint.clone());

    // setup router
    let _router = builder
        .accept(BLOBS_ALPN, BlobsProtocol::new(&blobs, None))
        .accept(GOSSIP_ALPN, gossip)
        .accept(DOCS_ALPN, docs.clone())
        .spawn();

    // let doc = match ticket {
    //         None =>docs.create().await?,
    //         Some(ticket) => {
    //             let ticket = DocTicket::from_str(&ticket)?;
    //             iroh.docs.import(ticket).await?
    //         }
    //     };
    // let mut list=docs.list().await?;
    // println!("{:?}",list.try_next().await?);
    // let doc =  match list.try_next().await? {
    //     Some((id,cap)) => docs.open(id).await.unwrap().unwrap(),
    //     None => docs.create().await?
    // };
    let doc = docs.create().await?;
    println!("getting blobs list");
    let hashes: Vec<Hash> = blobs.list().hashes().await?;
    for hash in hashes {
        let size = match blobs.status(hash).await? {
            iroh_blobs::api::proto::BlobStatus::Complete { size } => size,
            _ => continue,
        };
        let key = format!("notes/{}", hash.to_hex()); // whatever key scheme you want
        doc.set_hash(author, key, hash, size).await?;
        let data=blobs.get_bytes(hash).await?;
        println!("{:?}",data);
        println!("linked {} ({} bytes)", hash.to_hex(), size);
    }
    let ticket = doc.share(ShareMode::Write, Default::default()).await?;
    let mut stream = doc.subscribe().await?;
    println!("We are now serving {}", ticket);
    while let Some(n) = stream.next().await {
        println!("{:?}", n);
    }

    //do fun stuff with docs!
    Ok(())
}
// let hashes: Vec<Hash> = store.blobs().list().hashes().await?;
// let doc = docs.open(namespace_id).await?.unwrap();

// for hash in hashes {
//     let size = match store.blobs().status(hash).await? {
//         iroh_blobs::api::proto::BlobStatus::Complete { size } => size,
//         _ => continue,
//     };
//     let key = format!("notes/{}", hash.to_hex());  // whatever key scheme you want
//     doc.set_hash(author_id, key, hash, size).await?;
//     println!("linked {} ({} bytes)", hash.to_hex(), size);
// }

// let ticket = doc.share(ShareMode::Write, Default::default()).await?;
// println!("serving {}", ticket);
