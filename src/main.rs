extern crate blockchain;
extern crate sha2;

use blockchain::blockchain_imut::Blockchain;

fn main() {
    let bc: Blockchain<u8, ::sha2::Sha256> = Blockchain::new();
    // println!("valid: {:?}", bc.validate_chain());
    // {
    let bc = bc.append(5, 0);
    // println!("first block: {:?}", block);
    // println!("first hash: {:?}", block.hash());
    // }
    // {
    let bc = bc.append(10, 0);
    // println!("second block: {:?}", block);
    // println!("second hash: {:?}", block.hash());
    // }
    //
    bc.persist_to_disk("foobar").unwrap();
    let bc2: Blockchain<u8, ::sha2::Sha256> = Blockchain::load_from_disk("foobar").unwrap();
    assert_eq!(bc, bc2);
}
