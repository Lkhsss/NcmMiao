#[cfg(test)]
mod tests {
    use crate::*;
    
    #[test]
    #[ignore = "测验成功"]
    fn it_works() {
        let p = threadpool::Pool::new(4);
        p.execute(|| println!("do new job1"));
        p.execute(|| println!("do new job2"));
        p.execute(|| println!("do new job3"));
        p.execute(|| println!("do new job4"));
    }
}