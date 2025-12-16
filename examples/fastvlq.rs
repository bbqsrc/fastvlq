fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let n = match args.first().and_then(|x| x.parse::<u64>().ok()) {
        Some(v) => v,
        None => {
            eprintln!("Usage: <number>");
            std::process::exit(1);
        }
    };

    let v_be = fastvlq::encode_vu64_be(n);
    let v_le = fastvlq::encode_vu64_le(n);
    println!("BE: {:?}", v_be);
    println!("LE: {:?}", v_le);
}
