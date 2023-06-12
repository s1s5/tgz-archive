use tgz_archive::TgzArchive;

#[derive(TgzArchive)]
#[tgz_archive(path = "./assets", gzip = "auto")]
struct Assets;

#[derive(TgzArchive)]
#[tgz_archive(path = "./assets", gzip = "never")]
struct RawAssets;

#[derive(TgzArchive)]
#[tgz_archive(path = "./assets", gzip = "all")]
struct GzippedAssets;

fn main() {
    // ---- content, is_gzipped ---
    let key = String::from("a.txt");
    println!("{:?}", Assets::get(&key));
    // >> Some(([104, 101, 108, 108, 111, 10], false))

    // ---- raw content ---
    println!("{:?}", RawAssets::get("js/bar.js"));
    // >> Some([102, 111, 111, 10])

    // ---- always gzipped content ---
    println!("{:?}", GzippedAssets::get("js/bar.js"));
    // >> Some([31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 75, 203, 207, 231, 2, 0, 168, 101, 50, 126, 4, 0, 0, 0])
}
