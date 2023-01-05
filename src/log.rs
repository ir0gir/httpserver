use ansi_term::Colour::Red;

pub fn exit_msg(msg: &str) {
    println!("{}", Red.paint(msg));
    std::process::exit(1);
}