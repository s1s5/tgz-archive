use include_tgz::TgzArchive;

#[derive(TgzArchive)]
#[tgz_archive(path = "./assets")]
struct Assets {}

fn main() {
    println!("{:?}", Assets::get("a.txt"));
}
