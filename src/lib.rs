#[warn(dead_code)]
use aes::cipher::generic_array::typenum::U16;
use aes::cipher::{generic_array::GenericArray, BlockDecrypt, KeyInit};
use aes::Aes128;
use base64;
use colored::*;
use core::panic;
use log::{debug, error, info, trace, warn};
use serde_derive::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::fs::{self, File};
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom, Write};
// use std::iter::Enumerate;
use std::path::Path;
use std::str::from_utf8;

#[derive(Debug)]
pub struct Ncmfile {
    /// 文件对象
    pub file: File,
    /// 只是名称，不带后缀
    pub name: String,
    /// 带后缀名
    pub filename: String,
    /// 文件大小
    pub size: u64,
    /// 游标
    pub position: u64,
}
impl Ncmfile {
    pub fn new(filepath: &str) -> Result<Ncmfile, Error> {
        let file = File::open(filepath)?;
        let path = Path::new(filepath);
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        let size = file.metadata().unwrap().len();
        let name = match Path::new(&filepath).file_stem() {
            Some(f) => f.to_str().unwrap().to_string(),
            None => panic!("获取文件名失败"),
        };
        Ok(Ncmfile {
            file,
            name,
            filename,
            size,
            position: 0,
        })
    }
    /// 根据传入的长度来读取文件
    ///
    /// 该函数可以记录上次读取的位置，下次读取时从上次读取的位置开始
    /// - length 想要读取的长度
    pub fn seekread(&mut self, length: u64) -> Result<Vec<u8>, std::io::Error> {
        if self.position + length > self.size {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "无法读取！读取长度大于剩余文件大小！",
            ));
        } else {
            let mut reader = BufReader::new(&self.file);
            reader.seek(SeekFrom::Start(self.position))?;
            let mut buf = vec![0; length as usize];
            reader.read_exact(&mut buf)?;
            self.position += length;
            Ok(buf[..].to_vec())
        }
    }
    /// 从指定位置开始读取。
    ///
    /// ！！！该函数仍然会更新游标
    ///
    /// - offset 开始位置
    /// - length 想要读取的长度
    pub fn seekread_from(&mut self, offset: u64, length: u64) -> Result<Vec<u8>, std::io::Error> {
        if self.position + length > self.size {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "无法读取！读取长度大于剩余文件大小！",
            ));
        } else {
            let mut reader = BufReader::new(&self.file);
            reader.seek(SeekFrom::Start(offset))?;
            let mut buf = vec![0; length as usize];
            reader.read_exact(&mut buf)?;
            self.position = offset + length;
            Ok(buf[..].to_vec())
        }
    }
    pub fn seekread_to_end(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut reader = BufReader::new(&self.file);
        reader.seek(SeekFrom::Start(self.position))?;
        let mut buf = vec![0; self.size as usize - self.position as usize];
        reader.read_exact(&mut buf)?;
        self.position = self.size;
        Ok(buf[..].to_vec())
    }
    pub fn seekread_no_error(&mut self, length: u64) -> Vec<u8> {
        if self.position + length > self.size {
            if self.position >= self.size {
                return vec![];
            } else {
                let mut reader = BufReader::new(&self.file);
                let _ = reader.seek(SeekFrom::Start(self.position));

                let mut buf: Vec<u8> = vec![0; (self.size - self.position) as usize];
                let _ = reader.read_exact(&mut buf);
                self.position += length;
                return buf[..].to_vec();
            }
        } else {
            let mut reader = BufReader::new(&self.file);
            let _ = reader.seek(SeekFrom::Start(self.position));
            let mut buf = vec![0; length as usize];
            let _ = reader.read_exact(&mut buf);
            self.position += length;
            buf[..].to_vec()
        }
    }
    /// 跳过某些数据
    pub fn skip(&mut self, length: u64) -> Result<(), std::io::Error> {
        if self.position + length > self.size {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "无法跳过！跳过长度大于剩余文件大小！",
            ));
        } else {
            self.position += length;
            Ok(())
        }
    }
    ///按字节进行0x64异或。
    fn parse_key(key: &mut [u8]) -> &[u8] {
        for i in 0..key.len() {
            key[i] ^= 0x64;
        }
        key
    }
}

/// 存储元数据的结构体
#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    //编号
    #[serde(rename = "musicId", skip)] //没用过，跳过
    music_id: String,
    // 音乐名称
    #[serde(rename = "musicName")]
    music_name: String,
    // 艺术家
    #[serde(rename = "artist")]
    music_artist: Vec<(String, String)>,
    // 专辑id
    #[serde(rename = "albumId")]
    album_id: String,
    // 专辑
    #[serde(rename = "album")]
    album: String,
    //
    #[serde(rename = "albumPicDocId", skip)]
    album_pic_doc_id: String,
    //
    #[serde(rename = "albumPic", skip)]
    album_pic: String,
    // 比特率
    #[serde(rename = "bitrate")]
    bitrate: u128,
    //
    #[serde(rename = "mp3DocId", skip)]
    mp3_doc_id: String,
    // 时间长短
    #[serde(rename = "duration")]
    duration: u128,
    //
    #[serde(rename = "mvId")]
    mv_id: String,
    // 别名
    #[serde(rename = "alias")]
    alias: Vec<String>,
    // 译名
    #[serde(rename = "transNames")]
    trans_names: Vec<String>,
    // 音乐格式
    #[serde(rename = "format")]
    format: String,
}

// 存储各种密钥的结构体
pub struct Key {
    pub core: Vec<u8>,
    pub meta: Vec<u8>,
}

/// 解密函数
pub fn dump(ncmfile: &mut Ncmfile, keys: &Key, outputdir: &Path) -> Result<(), MyError> {
    info!("开始解密[{}]文件", ncmfile.filename.yellow());
    // 获取magic header 。应为CTENFDAM
    trace!("获取 magic header");
    let magic_header = match ncmfile.seekread(8) {
        Ok(header) => header,
        Err(_e) => {
            error!("读取magic header失败");
            return Err(MyError::MagicHeaderError);
        }
    };
    // 判断是否为ncm格式的文件
    match from_utf8(&magic_header) {
        Ok(header) => {
            if header != "CTENFDAM" {
                // 传播错误至dump
                return Err(MyError::MagicHeaderError);
            } else {
                trace!("[{}]为ncm格式文件", ncmfile.filename.yellow());
            }
        }
        // 传播错误至dump
        Err(_e) => return Err(MyError::MagicHeaderError),
    }

    // 跳过2字节
    trace!("跳过2字节");
    match ncmfile.skip(2) {
        Ok(_) => (),
        Err(_e) => return Err(MyError::FileSkipError),
    };

    trace!("获取RC4密钥长度");
    //小端模式读取RC4密钥长度 正常情况下应为128
    let key_length = u32::from_le_bytes(ncmfile.seekread(4).unwrap().try_into().unwrap()) as u64;
    debug!("RC4密钥长度为：{}", key_length);

    //读取密钥 开头应为 neteasecloudmusic
    trace!("读取RC4密钥");
    let mut key_data = ncmfile.seekread(key_length).unwrap();
    //aes128解密
    let key_data = &aes128_to_slice(&keys.core, Ncmfile::parse_key(&mut key_data[..])); //先把密钥按照字节进行0x64异或
                                                                                        // RC4密钥
    let key_data = unpad(&key_data[..])[17..].to_vec();

    //读取meta信息的数据大小
    trace!("获取meta信息数据大小");
    let meta_length = u32::from_le_bytes(ncmfile.seekread(4).unwrap().try_into().unwrap()) as u64;

    // 读取meta信息
    trace!("读取meta信息");
    let meta_data = {
        let mut meta_data = ncmfile.seekread(meta_length).unwrap(); //读取源数据
                                                                    //字节对0x63进行异或。
        for i in 0..meta_data.len() {
            meta_data[i] ^= 0x63;
        }
        // base64解密
        let decode_data = &base64::decode(&meta_data[22..]).unwrap()[..];
        // aes128解密
        let aes_data = aes128_to_slice(&keys.meta, decode_data);
        // unpadding
        let json_data = String::from_utf8(unpad(&aes_data)[6..].to_vec()).unwrap();
        debug!("json_data: {}", json_data);
        let data: Value = serde_json::from_str(&json_data[..]).unwrap(); //解析json数据
        data
    };

    // 跳过4个字节的校验码
    trace!("跳过4个字节的校验码");
    let crc32 = u32::from_le_bytes(ncmfile.seekread(4).unwrap().try_into().unwrap()) as u64;

    // 跳过5个字节
    trace!("跳过5个字节");
    ncmfile.skip(5).unwrap();

    // 获取图片数据的大小
    trace!("获取图片数据的大小");
    let image_data_length =
        u32::from_le_bytes(ncmfile.seekread(4).unwrap().try_into().unwrap()) as u64;

    // 读取图片，并写入文件当中
    trace!("暂不需要保存图片,跳过{}字节", image_data_length);
    let image_data = ncmfile.seekread(image_data_length).unwrap(); //读取图片数据
                                                                   // let _ = ncmfile.skip(image_data_length); //暂不需要保存图片,直接跳过这些字节就好

    //保存图片
    // trace!("保存图片");
    // let mut file = File::create(format!("TEST.jpg",)).unwrap();
    // file.write_all(&image_data).unwrap();

    let key_box = {
        let key_length = key_data.len();
        let key_data = Vec::from(key_data);
        let mut key_box = (0..=255).collect::<Vec<u8>>();
        let mut temp = 0;
        let mut last_byte = 0;
        let mut key_offset = 0;

        for i in 0..=255 {
            let swap = key_box[i as usize] as u64;
            temp = (swap + last_byte as u64 + key_data[key_offset as usize] as u64) & 0xFF;
            key_offset += 1;
            if key_offset >= key_length {
                key_offset = 0;
            }
            key_box[i as usize] = key_box[temp as usize];
            key_box[temp as usize] = swap as u8;
            last_byte = temp;
        }
        let key_box = key_box.clone();
        key_box
    };

    /* let mut s_box = {
        let key_length = key_data.len();
        let key_box = Vec::from(key_data);
        let mut s = (0..=255).collect::<Vec<u8>>();
        let mut j = 0;

        for i in 0..=255 {
            j = (j as usize + s[i] as usize + key_box[i % key_length] as usize) & 0xFF;

            //记录 s[j]的值
            let temp = &s.get(j as usize).unwrap().to_owned();
            s[j as usize] = s[i];
            s[i] = temp.to_owned();
        }

        s
    }; */

    // let key_box = key_box[0..(key_box.len()-key_box[key_box.len() as usize-1] as usize)].to_vec();

    //解密音乐数据
    trace!("解密音乐数据");
    let mut music_data: Vec<u8> = Vec::new();
    loop {
        let mut chunk = ncmfile.seekread_no_error(0x8000);

        let chunk_length = chunk.len();
        if chunk_length != 0 {
            for i in 1..chunk_length + 1 {
                let j = i & 0xFF;

                chunk[i - 1] ^= key_box[(key_box[j] as usize
                    + key_box[(key_box[j as usize] as usize + j) & 0xff] as usize)
                    & 0xff]
                // chunk[i - 1] ^= key_box[(key_box[j] + key_box[(key_box[j as usize] as usize + j as usize) & 0xFF]) & 0xFF];
            }
            //向music_data中最追加chunk
            music_data.append(&mut chunk);
        } else {
            break;
        }
    }

    //组成流密钥
    /* let mut stream = Vec::new();
    for i in 0..256 {
        stream.push(
            s_box[(s_box[i] as usize + s_box[(i + s_box[i] as usize) & 0xFF] as usize) & 0xFF],
        )
    } */

    // 解密音乐数据
    /* loop {
        let chunk = ncmfile.seekread_no_error(256); //每次读取256个字节
        if chunk.len() != 0 {
            for (count, &i) in chunk[..].iter().enumerate() {
                music_data.push(i ^ stream[count])
            }
        } else {
            break;
        }
    } */
    // debug!("music_data：{:?}", music_data);
    // debug!("长度：{}", stream.len());

    //退出循环，写入文件

    //处理文件路径
    trace!("拼接文件路径");
    let path = {
        let mut artists = String::new();
        let mut music_artist = Vec::new();
        if let Some(i) = meta_data.get("artist") {
            for names in i.as_array().unwrap() {
                music_artist.push(names[0].as_str().unwrap());
            }
        }

        for artist in music_artist {
            artists.push_str(&artist);
            artists.push(',');
        }
        // 移除最后一个字符
        artists.pop();
        debug!("艺术家名称：{}", artists.yellow());

        let filename = format!(
            "{} - {}.{}",
            meta_data.get("musicName").unwrap().as_str().unwrap(),
            artists,
            meta_data.get("format").unwrap().as_str().unwrap()
        );
        // .replace("\"", "＂")
        // .replace("?", "？")
        // .replace(":", "：");

        let filename = standardize_filename(filename);
        debug!("文件名：{}", filename.yellow());
        //链级创建输出目录
        fs::create_dir_all(outputdir).unwrap();
        outputdir.join(filename)
    };

    debug!("文件路径: {:?}", path);
    // 创建文件
    trace!("创建文件");
    let mut music_file = File::create(&path).unwrap();
    trace!("保存文件");
    music_file.write_all(&music_data).unwrap();
    // 关闭文件
    music_file.sync_all().unwrap();
    music_file.flush().unwrap();

    info!(
        "[{}]文件已保存到: {}",
        ncmfile.name.yellow(),
        path.to_str().unwrap().bright_cyan()
    );
    info!(
        "{}{}{}",
        "[".bright_green(),
        ncmfile.filename.yellow(),
        "]解密成功".bright_green()
    );
    Ok(())
}

// fn read_meta(file: &mut File, meta_length: u32) -> Result<Vec<u8>, Error> {}
fn convert_to_generic_arrays(input: &[u8]) -> Vec<GenericArray<u8, U16>> {
    // 确保输入的长度是16的倍数
    assert!(
        input.len() % 16 == 0,
        "Input length must be a multiple of 16"
    );

    input
        .chunks(16)
        .map(|chunk| {
            // 将每个块转换为GenericArray
            GenericArray::clone_from_slice(chunk)
        })
        .collect()
}
/// aes128解密
/// ！！！未对齐数据！！！
/// TODO
/// 解密NCM文件的rc4密钥前记得按字节对0x64进行异或
fn aes128(key: &[u8], blocks: &[u8]) -> String {
    trace!("进行AES128解密");
    let key = GenericArray::from_slice(key);

    let mut blocks = convert_to_generic_arrays(blocks);

    // 初始化密钥
    let cipher = Aes128::new(&key);

    // 开始解密
    cipher.decrypt_blocks(&mut blocks);

    let mut x = String::new();
    for block in blocks.iter() {
        x.push_str(std::str::from_utf8(&block).unwrap())
    }
    // 去除所有空格及控制字符
    let x = x[..].trim();

    x.to_string()
}
fn aes128_to_slice(key: &[u8], blocks: &[u8]) -> Vec<u8> {
    trace!("进行AES128解密");
    let key = GenericArray::from_slice(key);

    let mut blocks = convert_to_generic_arrays(blocks);

    // 初始化密钥
    let cipher = Aes128::new(&key);

    // 开始解密
    cipher.decrypt_blocks(&mut blocks);
    let mut x: Vec<u8> = Vec::new();
    for block in blocks.iter() {
        for i in block {
            x.push(i.to_owned());
        }
    }
    x
}

/// ## 规范文件名称
/// 防止创建文件失败
/// 符号一一对应：
/// -  \  /  *  ?  "  :   <  >  |
/// -  _  _  ＊  ？ ＂  ：  ⟨  ⟩   _
fn standardize_filename(old_filename: String) -> String {
    let mut new_filename = String::from(old_filename);
    debug!("规范文件名：{}", new_filename);
    let standard =   ["\\", "/", "*", "?", "\"", ":", "<", ">", "|"];
    let resolution = ["_", "_", "＊", "？", "＂", "：", "⟨", "⟩", "_",];
    for i in 0..standard.len() {
        new_filename = new_filename.replace(&standard[i].to_string(), &resolution[i].to_string());
    }
    new_filename
}

/// 使用PKCS5Padding标准，去掉填充信息
fn unpad(data: &[u8]) -> Vec<u8> {
    data[..data.len() - data[data.len() - 1] as usize].to_vec()
}

#[derive(Debug)]
pub enum MyError {
    MagicHeaderError,
    FileReadError,
    FileSkipError,
    FileWriteError,
    FilenameError,
    FileNotFoundError,
}

impl std::error::Error for MyError {}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::MagicHeaderError => write!(f, "文件不为NCM格式"),
            Self::FileReadError => write!(f, "文件读取错误"),
            Self::FileWriteError => write!(f, "文件写入错误"),
            Self::FilenameError => write!(f, "文件名不符合规范"),
            _ => write!(f, "未知错误"),
        }
    }
}
