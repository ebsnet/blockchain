use std::io::Read;

pub fn ask(msg: &str) -> bool {
    let mut inp = [b'0'];
    let mut stdin = ::std::io::stdin();
    println!("{}? [y/n]", msg);
    loop {
        if let Ok(_) = stdin.read_exact(&mut inp[..]) {
            match inp[0] {
                b'y' | b'Y' => return true,
                b'n' | b'N' => return false,
                _ => continue,
            }
        } else {
            return false;
        }
    }
}
