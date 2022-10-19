fn report(line: usize, wher: &str, msg: &str) {
    println!("[line {}] Error {} where: {}", line, wher, msg);
}

pub fn error(line: usize, msg:&str) {
    report(line, "", msg);
}