use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition, TableHandle, TypeName};

/// A raw-bytes key/value wrapper that matches a given iroh-blobs `type_name()`.
/// `SelfType` is `&[u8]` so we get the raw stored bytes.
#[derive(Debug)]
struct RawVal;
#[derive(Debug)]
struct RawKey;

impl redb::Value for RawVal {
    type SelfType<'a> = &'a [u8] where Self: 'a;
    type AsBytes<'a> = &'a [u8] where Self: 'a;
    fn fixed_width() -> Option<usize> { None }
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a { data }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> where Self: 'a, Self: 'b { value }
    fn type_name() -> TypeName { TypeName::new("raw") }
}

// ---- Hash (iroh_blobs::Hash): fixed 32 bytes ----
#[derive(Debug)]
struct HashT;
impl redb::Value for HashT {
    type SelfType<'a> = [u8; 32] where Self: 'a;
    type AsBytes<'a> = &'a [u8; 32] where Self: 'a;
    fn fixed_width() -> Option<usize> { Some(32) }
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a {
        data.try_into().unwrap()
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> where Self: 'a, Self: 'b { value }
    fn type_name() -> TypeName { TypeName::new("iroh_blobs::Hash") }
}
impl redb::Key for HashT {
    fn compare(a: &[u8], b: &[u8]) -> std::cmp::Ordering { a.cmp(b) }
}

// ---- EntryState (variable, postcard) ----
#[derive(Debug)]
struct EntryStateT;
impl redb::Value for EntryStateT {
    type SelfType<'a> = &'a [u8] where Self: 'a;
    type AsBytes<'a> = &'a [u8] where Self: 'a;
    fn fixed_width() -> Option<usize> { None }
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a { data }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> where Self: 'a, Self: 'b { value }
    fn type_name() -> TypeName { TypeName::new("EntryState") }
}

// ---- HashAndFormat (fixed 33 bytes, postcard) ----
#[derive(Debug)]
struct HashAndFormatT;
impl redb::Value for HashAndFormatT {
    type SelfType<'a> = [u8; 33] where Self: 'a;
    type AsBytes<'a> = &'a [u8; 33] where Self: 'a;
    fn fixed_width() -> Option<usize> { Some(33) }
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a {
        data.try_into().unwrap()
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> where Self: 'a, Self: 'b { value }
    fn type_name() -> TypeName { TypeName::new("iroh_blobs::HashAndFormat") }
}

// ---- Tag (variable bytes) ----
#[derive(Debug)]
struct TagT;
impl redb::Value for TagT {
    type SelfType<'a> = &'a [u8] where Self: 'a;
    type AsBytes<'a> = &'a [u8] where Self: 'a;
    fn fixed_width() -> Option<usize> { None }
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a { data }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> where Self: 'a, Self: 'b { value }
    fn type_name() -> TypeName { TypeName::new("Tag") }
}
impl redb::Key for TagT {
    fn compare(a: &[u8], b: &[u8]) -> std::cmp::Ordering { a.cmp(b) }
}

fn hex(b: &[u8]) -> String { hex_encode(b) }
fn hex_encode(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for byte in b { s.push_str(&format!("{:02x}", byte)); }
    s
}

fn show_text(label: &str, bytes: &[u8]) {
    match std::str::from_utf8(bytes) {
        Ok(s) => {
            let clean = s.chars().all(|c| !c.is_control() || c == '\n' || c == '\r' || c == '\t');
            if clean {
                println!("{} [{} bytes, text]:\n{}", label, bytes.len(), s);
            } else {
                println!("{} [{} bytes, mixed]: {:02x?}", label, bytes.len(), bytes);
            }
        }
        Err(_) => {
            println!("{} [{} bytes, binary]: {:02x?}", label, bytes.len(), &bytes[..bytes.len().min(96)]);
        }
    }
}

fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "/home/jjj/HACKING/sharednotescli/blobs.db".to_string()
    });
    let db = Database::open(&path)?;

    let read_txn = db.begin_read()?;

    println!("===== tables =====");
    for t in read_txn.list_tables()? {
        println!("  {}", t.name());
    }

    // tags-0: Tag -> HashAndFormat
    println!("\n===== tags-0 (Tag -> HashAndFormat) =====");
    {
        let def: TableDefinition<TagT, HashAndFormatT> = TableDefinition::new("tags-0");
        if let Ok(table) = read_txn.open_table(def) {
            for entry in table.iter()? {
                let (k, v) = entry?;
                let name = String::from_utf8_lossy(k.value());
                let raw = v.value();
                let hash = &raw[..32];
                let format = raw[32];
                let fmt = match format { 0 => "Raw", 1 => "HashSeq", _ => "Unknown" };
                println!("  tag {:?} => hash={} format={} ({})", name, hex(hash), format, fmt);
            }
        } else {
            println!("  (could not open)");
        }
    }

    // blobs-0: Hash -> EntryState
    println!("\n===== blobs-0 (Hash -> EntryState) =====");
    {
        let def: TableDefinition<HashT, EntryStateT> = TableDefinition::new("blobs-0");
        if let Ok(table) = read_txn.open_table(def) {
            for entry in table.iter()? {
                let (k, v) = entry?;
                let key = k.value();
                let val = v.value();
                // EntryState is postcard. Variant tag is first byte: 0=Complete, 1=Partial.
                let variant = val.first().copied();
                let (vname, extra) = match variant {
                    Some(0) => ("Complete", decode_complete(&val[1..])),
                    Some(1) => ("Partial", decode_partial(&val[1..])),
                    _ => ("?", String::new()),
                };
                println!("  hash {} => {}{}", hex(&key), vname, extra);
            }
        } else {
            println!("  (could not open)");
        }
    }

    // inline-data-0: Hash -> bytes  (the actual blob content for inline blobs)
    println!("\n===== inline-data-0 (Hash -> data) =====");
    {
        let def: TableDefinition<HashT, &[u8]> = TableDefinition::new("inline-data-0");
        if let Ok(table) = read_txn.open_table(def) {
            for entry in table.iter()? {
                let (k, v) = entry?;
                let key = k.value();
                let val = v.value();
                println!("------------------------------------------------------------");
                println!("hash {}", hex(&key));
                show_text("data", val);
            }
        } else {
            println!("  (could not open)");
        }
    }

    // inline-outboard-0: Hash -> bytes
    println!("\n===== inline-outboard-0 (Hash -> outboard) =====");
    {
        let def: TableDefinition<HashT, &[u8]> = TableDefinition::new("inline-outboard-0");
        if let Ok(table) = read_txn.open_table(def) {
            let mut count = 0;
            for entry in table.iter()? {
                let (k, v) = entry?;
                let key = k.value();
                let val = v.value();
                println!("  hash {} => outboard {} bytes", hex(&key), val.len());
                count += 1;
            }
            if count == 0 { println!("  (empty)"); }
        } else {
            println!("  (could not open)");
        }
    }

    Ok(())
}

// Best-effort decoding of postcard-serialized EntryState::Complete payload
// (data_location, outboard_location). We only summarize sizes/locations.
fn decode_complete(data: &[u8]) -> String {
    // postcard enum DataLocation: 0=Inline, 1=Owned, 2=External
    // postcard enum OutboardLocation: 0=Inline, 1=Owned, 2=NotNeeded
    // Layout depends on exact serde variant order; show raw bytes for safety.
    format!(" {{ raw: {:02x?} }}", data)
}

fn decode_partial(data: &[u8]) -> String {
    format!(" {{ raw: {:02x?} }}", data)
}
