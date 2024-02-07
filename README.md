# NcmMiao :tada:
一个使用Rust语言编写的ncm文件解密工具。

### 功能及特点
 - 支持单一文件，多文件夹递归批量解密。
 - 完善的日志功能
 - Colorful
 - 编译文件小，解密快

## 编译
```
cargo build -r
```

## 使用
支持单一文件，多文件夹递归批量解密。
```
cargo build -r <文件或文件夹路径1> <文件或文件夹路径2> <文件或文件夹路径3> ... 
```
> 注意！如果没有指定任何文件夹或者文件，那么程序将自动读取工作目录下的CloudMusic和input文件夹下的文件。如果都没有将自动退出。

输出文件夹在output。等我想写了再写命令行解析（bushi。

---

# TODO :construction:
 - [ ] 增加多线程支持
 - [ ] 增加解密进度条
 - [ ] 增加命令行解析
 - [ ] 增加自定义输出文件夹

---

# CHANGELOG
## [1.0.0] - 2024-1-27
### Features :sparkles:
- 初步完成解密函数。主程序成型

## [1.1.1] - 2024-2-1
### Features :sparkles:
- 完成批量解密
### Fixed :bug:
- 修正了提取音乐信息会有数据类型错误导致panic的问题

## [1.1.2] - 2024-2-5
### Fixed :bug:
 - 修正了提取音乐信息时，部分歌曲信息提取失败的问题
 - 修正了音乐数据解密失败的问题
## [1.1.3] - 2024-2-6
### Fixed :bug:
 - 修正了部分音乐名称中含有不合法字符时创建文件失败的问题

---

# 附 - ncm文件结构
|信息|大小|作用|
|:-:|:-:|:-:|
|Magic Header|8 bytes|文件头|
|Gap|2 bytes||
|Key Length|4 bytes|RC4密钥长度，字节是按小端排序。|
|Key Data|Key Length|RC4密钥|
|Music Info Length|4 bytes|用AES128加密后的音乐相关信息的长度，小端排序。|
|Music Info Data|Music Info Length|Json格式音乐信息数据。|
|Gap|5 bytes||
|CRC校验码|4 bytes|图片的CRC32校验码，小端排序。|
|Image Size|4 bytes|图片的大小|
|Image Data|Image Size|图片数据|
|Music Data||音乐数据|
---
### Magic Header
### Key Data
用AES128加密后的RC4密钥。
1. 先按字节对0x64进行异或。
2. AES解密,去除填充部分。
3. 去除最前面'neteasecloudmusic'17个字节，得到RC4密钥。
### Music Info Data
Json格式音乐信息数据。
1. 按字节对0x63进行异或。
2. 去除最前面22个字节。
3. Base64进行解码。
4. AES解密。
6. 去除前面6个字节，后面数量为最后一个字节的字节数的垃圾数据，得到Json数据。

### Music Data
1. RC4-KSA生成S盒。
2. 用S盒解密(自定义的解密方法)，不是RC4-PRGA解密。


