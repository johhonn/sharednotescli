use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition, TableHandle};

fn hex(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for byte in b { s.push_str(&format!("{:02x}", byte)); }
    s
}

fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "/home/jjj/HACKING/sharednotescli/docs.redb".to_string()
    });
    let db = Database::open(&path)?;
    let tx = db.begin_read()?;

    println!("== tables ==");
    for t in tx.list_tables()? { println!("  {}", t.name()); }

    // authors-1: [u8;32] -> [u8;32]
    println!("\n== authors-1 ==");
    let a: TableDefinition<&[u8;32], &[u8;32]> = TableDefinition::new("authors-1");
    if let Ok(tab) = tx.open_table(a) {
        for e in tab.iter()? {
            let (k,v) = e?;
            println!("  {} => {}", hex(k.value()), hex(v.value()));
        }
    } else { println!("  (open failed)"); }

    // namespaces-2: [u8;32] -> (u8, [u8;32])
    println!("\n== namespaces-2 ==");
    let n: TableDefinition<&[u8;32], (u8, &[u8;32])> = TableDefinition::new("namespaces-2");
    if let Ok(tab) = tx.open_table(n) {
        for e in tab.iter()? {
            let (k,v) = e?;
            let (kind, cap) = v.value();
            println!("  {} => kind={} cap={}", hex(k.value()), kind, hex(cap));
        }
    } else { println!("  (open failed)"); }

    // records-1: ([u8;32],[u8;32],&[u8]) -> (u64,[u8;64],[u8;64],u64,[u8;32])
    println!("\n== records-1 ==");
    let r: TableDefinition<(&[u8;32], &[u8;32], &[u8]), (u64, &[u8;64], &[u8;64], u64, &[u8;32])> =
        TableDefinition::new("records-1");
    match tx.open_table(r) {
        Ok(tab) => {
            let mut count = 0;
            for e in tab.iter()? {
                let (k,v) = e?;
                let (ns, author, key) = k.value();
                let (ts, _sn, _sa, len, hash) = v.value();
                let keystr = String::from_utf8_lossy(key);
                println!("  ns={} author={} key={:?} ts={} len={} hash={}",
                    hex(ns), hex(author), keystr, ts, len, hex(hash));
                count += 1;
            }
            println!("  ({} records)", count);
        }
        Err(e) => println!("  (open failed: {:?})", e),
    }

    Ok(())
}
