use std::path::Path;

fn main() {
    let path = Path::new("/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/sample_ext/07270143.SRW");
    let mime = tree_magic_mini::from_filepath(path);
    println!("SRW file detected as: {:?}", mime);
}
