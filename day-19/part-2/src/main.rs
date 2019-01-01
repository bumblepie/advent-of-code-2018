fn main() {
    let x = 10551309;
    let factor_sum: i32 = (1..x+1).filter(|n| x % n == 0).sum();
    println!("{}", factor_sum);
}
//
//[0, 10551309, 7, 10551309, 1, 1]
//
//X = 10551309
//COUNT = 0
//N = 1
//Z = 1
//loop {
//Y = 1
//loop {
//Y = N * Z
//if Y == X {
//COUNT += Z
//}
//N += 1
//if Y > X {
//break
//}
//}
//Z += 1
//if Z > COUNT {
//break
//}
//}
//
//ie sum all factors of X