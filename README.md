# tgz-archive

Include all asset files in the binary.

```rust
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
    println!("{:?}", Assets::get("a.txt"));
    // >> Some(([104, 101, 108, 108, 111, 10], false))

    // ---- raw content ---
    println!("{:?}", RawAssets::get("js/bar.js"));
    // >> Some([102, 111, 111, 10])

    // ---- always gzipped content ---
    println!("{:?}", GzippedAssets::get("js/bar.js"));
    // >> Some([31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 75, 203, 207, 231, 2, 0, 168, 101, 50, 126, 4, 0, 0, 0])
}
```


## features
- path

  - "path" must be specified as a relative path from project root directory.

- gzip
  - "never": Default value. This option means that gzip should never be used.
  - "auto": Selecting this option will enable gzip compression when it results in a smaller file size.
  - "all": This option instructs to apply gzip compression to all files.