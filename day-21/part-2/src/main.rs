fn main() {
    let mut x: usize = 0;
    let mut answers = Vec::new();

    loop {
        let mut y = x | 65536;
        x = 7586220;

        while y >= 256 {
            x = (((x + (y & 255)) & 16_777_215) * 65899) & 16777215;
            y /= 256;
        }
        x = (((x + (y & 255)) & 16_777_215) * 65899) & 16777215;

        if answers.contains(&x) {
            break;
        }
        answers.push(x);
    }
    println!("{:?}", answers);
    println!("{}", answers.last().unwrap());
}
