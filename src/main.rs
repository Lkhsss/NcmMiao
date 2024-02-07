use env_logger::Builder;
use hex::decode;
use log::{error, warn};
use ncmmiao::{dump, Key, Ncmfile};
use std::env;
use std::path::Path;
// use std::time::SystemTime;
use walkdir::WalkDir;
#[warn(unreachable_code)]

fn main() {
    let mut builder = Builder::new();
    builder.filter(None, log::LevelFilter::Info);
    builder.init(); //初始化logger

    let keys = Key {
        core: decode("687A4852416D736F356B496E62617857").unwrap(),
        meta: decode("2331346C6A6B5F215C5D2630553C2728").unwrap(),
    };

    let args: Vec<String> = env::args().collect();
    let args = if args.len() == 1 {
        warn!("未指定文件夹，将使用默认文件夹。");
        let mut args_temp = Vec::new();
        if Path::new("CloudMusic").exists() {
            warn!("CloudMusic文件夹存在，将自动使用。");
            args_temp.push(String::from("CloudMusic"));
        };
        if Path::new("input").exists() {
            warn!("input文件夹存在，将自动使用。");
            args_temp.push(String::from("input"));
        };
        if args_temp.is_empty() {
            //TODO 增加软件介绍
            error!("没有参数\n没有CloudMusic或者input文件夹存在与于工作目录");
            panic!("没有参数\n没有CloudMusic或者input文件夹存在与于工作目录");
        }
        args_temp
    } else {
        args[1..].to_vec()
    };
    // let args = &args[1..];

    let mut undumpfile = Vec::new(); // 该列表将存入文件的路径

    for arg in &args {
        //解析传入的每一个路径：文件or文件夹
        let path = Path::new(arg);

        if path.is_file() {
            // 当后缀符合为ncm时才加入列表
            match path.extension() {
                Some(extension) => {
                    if extension == "ncm" {
                        let _ = &mut undumpfile.push(arg.to_owned());
                    }
                }
                None => {}
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(path) {
                let new_entry = entry.unwrap().clone();
                let filepath = new_entry.into_path();
                // 当后缀符合为ncm时才加入列表
                match filepath.extension() {
                    Some(extension) => {
                        if extension == "ncm" {
                            let _ = &mut undumpfile.push(String::from(filepath.to_str().unwrap()));
                        }
                    }
                    None => {
                        continue;
                    }
                }
            }
        }
    }
    // let filepaths = undumpfile;
    let count = undumpfile.len();
    let mut time = 0usize;
    for filepath in undumpfile {
        time += 1;
        // println!("{}/{}", &time, &count);
        let mut ncmfile = Ncmfile::new(&filepath).unwrap();
        dump(&mut ncmfile, &keys, Path::new("output")).unwrap();

        //TODO增加计时
        // let time = SystemTime::now();
        // let time = SystemTime::now().duration_since(time).unwrap().as_secs();
        // println!("{}",time);
    }
}
