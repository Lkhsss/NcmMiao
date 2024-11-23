use clap::Parser;

#[derive(Parser)]
#[command(name = "ncmmiao")]
#[command(author = "lkhsss")]
#[command(version,about = "一个解密ncm文件的神秘程序 By Lkhsss", long_about = None)]
pub struct Cli {
    /// 并发的最大线程数，默认为4线程
    #[arg(short, long)]
    pub workers: Option<usize>,
    /// 需要解密的文件夹或文件
    #[arg(short, long, name = "输入文件/文件夹")]
    pub input: Vec<String>,

    #[arg(short, long, name = "输出文件夹", default_value = "NcmmiaoOutput")]
    pub output: Option<String>,
}
